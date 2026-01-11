// src-tauri/src/services/playback_service.rs
//
// Playback Service - Player Integration
//
// PHASE 6 IMPLEMENTATION COMPLETE
// - Launches MPV (if needed) with fixed IPC pipe
// - Loads file via IPC (replace mode)
// - Resumes from saved progress (>5 min threshold)
// - Controls pause/seek/stop via IPC commands
// - Observer handles progress/pause/eof events
// - All state changes go through events → EpisodeService persists
//
// PHASE 4 CORRECTIONS:
// - REMOVED: get_linked_files call (hallucinated API)
// - file_id is now required in StartPlaybackRequest

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::events::{EventBus, PlaybackProgressUpdated, PlaybackStarted};
use crate::integrations::mpv::MpvClient;
use crate::repositories::{EpisodeRepository, FileRepository};
use crate::services::playback_observer::{ObserverConfig, PlaybackObserver};

const PIPE_NAME: &str = "animehub-mpv";
const RESUME_THRESHOLD_SECONDS: u64 = 300;

#[derive(Debug, Clone)]
pub struct StartPlaybackRequest {
    pub episode_id: Uuid,
    /// CORRECTION: file_id is now required since get_linked_files doesn't exist
    pub file_id: Uuid,
}

fn send_ipc_command(json_cmd: &str) -> std::io::Result<()> {
    let pipe_path = format!(r"\\.\pipe\{}", PIPE_NAME);
    let mut file = std::fs::OpenOptions::new().write(true).open(pipe_path)?;
    file.write_all(json_cmd.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

pub struct PlaybackService {
    episode_repo: Arc<dyn EpisodeRepository>,
    file_repo: Arc<dyn FileRepository>,
    event_bus: Arc<EventBus>,
    mpv_client: Arc<MpvClient>,
    observer: Arc<PlaybackObserver>,
}

impl PlaybackService {
    pub fn new(
        episode_repo: Arc<dyn EpisodeRepository>,
        file_repo: Arc<dyn FileRepository>,
        event_bus: Arc<EventBus>,
        mpv_client: Arc<MpvClient>,
    ) -> Self {
        let observer = Arc::new(PlaybackObserver::new(
            mpv_client.clone(),
            event_bus.clone(),
            ObserverConfig::default(),
        ));

        Self {
            episode_repo,
            file_repo,
            event_bus,
            mpv_client,
            observer,
        }
    }

    pub fn start_playback(&self, request: StartPlaybackRequest) -> AppResult<PathBuf> {
        let episode = self
            .episode_repo
            .get_by_id(request.episode_id)?
            .ok_or(AppError::NotFound)?;

        // CORRECTION: file_id is now required, no fallback to get_linked_files
        let file = self
            .file_repo
            .get_by_id(request.file_id)?
            .ok_or(AppError::NotFound)?;

        if file.tipo != crate::domain::file::FileType::Video {
            return Err(AppError::Other("Not a video file".to_string()));
        }
        if !file.caminho_absoluto.exists() {
            return Err(AppError::Other("File not found on disk".to_string()));
        }

        let saved_progress = episode.progresso_atual;
        let start_pos = if saved_progress >= RESUME_THRESHOLD_SECONDS {
            saved_progress
        } else {
            0
        };

        // CORREÇÃO: is_running retorna bool
        let is_running = self.mpv_client.is_running();
        if !is_running {
            Command::new("mpv")
                .arg(format!("--input-ipc-server=\\\\.\\pipe\\{}", PIPE_NAME))
                .arg("--idle=yes")
                .arg("--keep-open=yes")
                .arg("--no-terminal")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| AppError::Other(format!("Failed to launch MPV: {}", e)))?;
        }

        let escaped_path = file
            .caminho_absoluto
            .to_string_lossy()
            .replace("\\", "\\\\")
            .replace("\"", "\\\"");
        let load_cmd = format!(r#"{{"command":["loadfile","{}","replace"]}}"#, escaped_path);
        let _ = send_ipc_command(&load_cmd);

        if start_pos > 0 {
            let seek_cmd = format!(r#"{{"command":["seek",{},"absolute"]}}"#, start_pos);
            let _ = send_ipc_command(&seek_cmd);
        }

        self.observer
            .start_observing(request.episode_id, start_pos, episode.duracao_esperada);

        self.event_bus
            .emit(PlaybackStarted::new(request.episode_id));

        Ok(file.caminho_absoluto.clone())
    }

    pub fn toggle_pause(&self) -> AppResult<()> {
        let cmd = r#"{"command":["cycle","pause"]}"#;
        send_ipc_command(cmd).map_err(|e| AppError::Other(e.to_string()))?;
        Ok(())
    }

    pub fn seek_to(&self, episode_id: Uuid, position_seconds: u64) -> AppResult<()> {
        let cmd = format!(r#"{{"command":["seek",{},"absolute"]}}"#, position_seconds);
        send_ipc_command(&cmd).map_err(|e| AppError::Other(e.to_string()))?;
        self.report_progress(episode_id, position_seconds)?;
        Ok(())
    }

    pub fn stop_playback(&self, episode_id: Uuid) -> AppResult<()> {
        if let Ok(pos) = self.mpv_client.get_position() {
            self.report_progress(episode_id, pos)?;
        }
        let cmd = r#"{"command":["quit"]}"#;
        let _ = send_ipc_command(cmd);
        self.observer.stop_observing();
        Ok(())
    }

    fn report_progress(&self, episode_id: Uuid, progress_seconds: u64) -> AppResult<()> {
        self.event_bus
            .emit(PlaybackProgressUpdated::new(episode_id, progress_seconds));
        Ok(())
    }

    // Keep get_current_position if needed elsewhere
    pub fn get_current_position(&self, episode_id: Uuid) -> AppResult<u64> {
        let episode = self
            .episode_repo
            .get_by_id(episode_id)?
            .ok_or(AppError::NotFound)?;
        Ok(episode.progresso_atual)
    }
}
