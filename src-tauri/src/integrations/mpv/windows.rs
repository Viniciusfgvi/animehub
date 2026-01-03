use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::process::{Command, Child};
use std::io::{Read, Write};

use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::Pipes::*;

use anyhow::{Result, anyhow};

const PIPE_NAME: &str = r"\\.\pipe\mpv-animehub";

pub struct MpvClient {
    process: Child,
    pipe: HANDLE,
}

impl MpvClient {
    pub fn launch(video: PathBuf) -> Result<Self> {
        let mut process = Command::new("mpv")
            .arg(video)
            .arg(format!("--input-ipc-server={}", PIPE_NAME))
            .spawn()
            .map_err(|e| anyhow!("Failed to start mpv: {}", e))?;

        // Espera MPV criar o pipe
        let pipe = loop {
            let wide: Vec<u16> = OsStr::new(PIPE_NAME)
                .encode_wide()
                .chain(once(0))
                .collect();

            unsafe {
                let handle = CreateFileW(
                    PCWSTR(wide.as_ptr()),
                    FILE_GENERIC_READ | FILE_GENERIC_WRITE,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    None,
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    None,
                );

                if handle != INVALID_HANDLE_VALUE {
                    break handle;
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(50));
        };

        Ok(Self { process, pipe })
    }

    pub fn send(&self, command: &str) -> Result<()> {
        let mut bytes = command.as_bytes().to_vec();
        bytes.push(b'\n');

        unsafe {
            let mut written = 0;
            WriteFile(
                self.pipe,
                Some(bytes.as_ptr() as _),
                bytes.len() as u32,
                Some(&mut written),
                None,
            )
            .ok()
            .map_err(|_| anyhow!("Failed to write to MPV pipe"))?;
        }

        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        self.send(r#"{"command":["set_property","pause",true]}"#)
    }

    pub fn resume(&self) -> Result<()> {
        self.send(r#"{"command":["set_property","pause",false]}"#)
    }

    pub fn seek(&self, seconds: i64) -> Result<()> {
        self.send(&format!(
            r#"{{"command":["seek",{}, "absolute"]}}"#,
            seconds
        ))
    }

    pub fn stop(&mut self) -> Result<()> {
        self.send(r#"{"command":["quit"]}"#)?;
        self.process.kill().ok();
        unsafe { CloseHandle(self.pipe) };
        Ok(())
    }
}
