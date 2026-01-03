// src-tauri/src/integrations/mpv/client.rs
//
// MPV Player Integration - Windows Implementation
//
// CRITICAL: Uses Windows Named Pipes for IPC as required.
// MPV launched with: --input-ipc-server=\\.\pipe\animehub-mpv
//
// WINDOWS API CONSTRAINTS (MANDATORY):
// - Use slices (&[u8], &mut [u8]) for WriteFile/ReadFile.
// - No raw pointers (as_ptr, as_mut_ptr).
// - No manual buffer lengths.
// - Synchronous IO (None for OVERLAPPED).

use std::path::{Path, PathBuf};
use std::process::{Command, Child, Stdio};
use std::sync::{Arc, Mutex};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use serde::{Deserialize, Serialize};
use serde_json::json;

// Windows API imports from the 'windows' crate
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HANDLE, CloseHandle, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, WriteFile, FILE_GENERIC_READ, FILE_GENERIC_WRITE,
    FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL,
};

use crate::error::{AppError, AppResult};

/// MPV IPC command envelope
#[derive(Debug, Serialize)]
struct MpvCommand {
    command: Vec<serde_json::Value>,
}

/// MPV IPC response envelope
#[derive(Debug, Deserialize)]
struct MpvResponse {
    error: String,
    #[serde(default)]
    data: Option<serde_json::Value>,
}

/// MPV Client for Windows
/// 
/// Handles process lifecycle and IPC communication.
/// Note: This client does not persist domain state or call services.
pub struct MpvClient {
    /// MPV process handle (Arc/Mutex for thread-safe access from commands)
    process: Arc<Mutex<Option<Child>>>,
    /// Predefined pipe name for AnimeHub
    pipe_name: String,
}

impl MpvClient {
    /// Creates a new instance of the MPV client.
    pub fn new() -> AppResult<Self> {
        Ok(Self {
            process: Arc::new(Mutex::new(None)),
            pipe_name: r"\\.\pipe\animehub-mpv".to_string(),
        })
    }

    /// Launches MPV with IPC enabled and starts playback.
    pub fn launch(&self, video_path: PathBuf) -> AppResult<PathBuf> {
        if !video_path.exists() {
            return Err(AppError::Other(format!("Video file not found: {:?}", video_path)));
        }

        // Cleanup existing process
        self.stop()?;

        let mut cmd = Command::new("mpv");
        cmd.arg(format!("--input-ipc-server={}", self.pipe_name))
            .arg("--idle=yes")
            .arg("--force-window=yes")
            .arg("--keep-open=yes")
            .arg(&video_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        let child = cmd.spawn()
            .map_err(|e| AppError::Other(format!("Failed to spawn MPV: {}", e)))?;

        {
            let mut proc_guard = self.process.lock().unwrap();
            *proc_guard = Some(child);
        }

        // Wait for MPV to create the pipe server
        std::thread::sleep(std::time::Duration::from_millis(600));

        Ok(video_path)
    }

    /// Stops playback and kills the MPV process.
    pub fn stop(&self) -> AppResult<()> {
        if self.is_running() {
            let _ = self.send_command(&["quit"]);
        }

        let mut proc_guard = self.process.lock().unwrap();
        if let Some(mut child) = proc_guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        Ok(())
    }

    /// Checks if MPV is currently active.
    pub fn is_running(&self) -> bool {
        let mut proc_guard = self.process.lock().unwrap();
        if let Some(ref mut child) = *proc_guard {
            match child.try_wait() {
                Ok(None) => true,
                _ => {
                    *proc_guard = None;
                    false
                }
            }
        } else {
            false
        }
    }

    // --- Media Control Commands ---

    pub fn pause(&self) -> AppResult<()> {
        self.send_command(&["set_property", "pause", "yes"])?;
        Ok(())
    }

    pub fn resume(&self) -> AppResult<()> {
        self.send_command(&["set_property", "pause", "no"])?;
        Ok(())
    }

    pub fn seek(&self, position_seconds: u64) -> AppResult<()> {
        self.send_command(&["seek", &position_seconds.to_string(), "absolute"])?;
        Ok(())
    }

    pub fn get_position(&self) -> AppResult<u64> {
        let response = self.send_command(&["get_property", "time-pos"])?;
        let pos = response.data
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        Ok(pos as u64)
    }

    pub fn get_duration(&self) -> AppResult<Option<u64>> {
        let response = self.send_command(&["get_property", "duration"])?;
        Ok(response.data.and_then(|v| v.as_f64()).map(|d| d as u64))
    }

    // --- IPC Implementation (Windows Named Pipes) ---

    /// Internal helper to communicate with MPV using the JSON IPC protocol.
    fn send_command(&self, command: &[&str]) -> AppResult<MpvResponse> {
        if !self.is_running() {
            return Err(AppError::Other("MPV is not running".to_string()));
        }

        // 1. Prepare Wide String for Windows API
        let pipe_path: Vec<u16> = OsStr::new(&self.pipe_name)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // 2. Open the Named Pipe
        let handle = unsafe {
            CreateFileW(
                PCWSTR(pipe_path.as_ptr()),
                FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None, // Security
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                HANDLE::default(),
            )
        }.map_err(|e| AppError::Other(format!("IPC Connection Error: {}", e)))?;

        if handle == INVALID_HANDLE_VALUE {
            return Err(AppError::Other("Invalid Pipe Handle".to_string()));
        }

        // Use RAII guard to ensure handle is closed
        let _guard = HandleGuard(handle);

        // 3. Serialize Command (MPV expects newline-delimited JSON)
        let cmd_json = MpvCommand {
            command: command.iter().map(|&s| json!(s)).collect(),
        };
        let mut payload = serde_json::to_string(&cmd_json)
            .map_err(|e| AppError::Serialization(e))?;
        payload.push('\n');

        // 4. Write to Pipe
        // Exactly as defined by windows crate: handle, Option<&[u8]>, Option<&mut u32>, Option<*mut OVERLAPPED>
        let mut written: u32 = 0;
        unsafe {
            WriteFile(
                handle,
                Some(payload.as_bytes()),
                Some(&mut written),
                None,
            )
        }.map_err(|e| AppError::Other(format!("IPC Write error: {}", e)))?;

        // 5. Read Response
        // Exactly as defined: handle, Option<&mut [u8]>, Option<&mut u32>, Option<*mut OVERLAPPED>
        let mut buffer = [0u8; 2048];
        let mut read: u32 = 0;
        unsafe {
            ReadFile(
                handle,
                Some(&mut buffer),
                Some(&mut read),
                None,
            )
        }.map_err(|e| AppError::Other(format!("IPC Read error: {}", e)))?;

        // 6. Parse and Validate
        let response_str = std::str::from_utf8(&buffer[..(read as usize)])
            .map_err(|_| AppError::Other("IPC response invalid UTF-8".to_string()))?;
        
        let response: MpvResponse = serde_json::from_str(response_str.trim())
            .map_err(|e| AppError::Serialization(e))?;

        if response.error != "success" {
            return Err(AppError::Other(format!("MPV IPC Error: {}", response.error)));
        }

        Ok(response)
    }
}

/// RAII Guard for Windows HANDLEs
struct HandleGuard(HANDLE);
impl Drop for HandleGuard {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe { let _ = CloseHandle(self.0); }
        }
    }
}

impl Drop for MpvClient {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

impl Default for MpvClient {
    fn default() -> Self {
        Self::new().expect("Critical: Failed to initialize MPV Client")
    }
}