// src-tauri/src/application/commands/playback_commands.rs

use tauri::State;
use uuid::Uuid;

use crate::application::state::AppState;
use crate::services::{PlaybackService, StartPlaybackRequest};

#[tauri::command]
pub async fn start_playback(
    state: State<'_, AppState>,
    episode_id: Uuid,
    file_id: Option<Uuid>,
) -> Result<String, String> {
    let playback_service: std::sync::Arc<PlaybackService> = state.playback_service.clone();

    let request = StartPlaybackRequest {
        episode_id,
        file_id,
    };

    let file_path = playback_service
        .start_playback(request)
        .map_err(|e| e.to_string())?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn toggle_pause_playback(state: State<'_, AppState>) -> Result<(), String> {
    let playback_service: std::sync::Arc<PlaybackService> = state.playback_service.clone();

    playback_service
        .toggle_pause()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn seek_playback(
    state: State<'_, AppState>,
    episode_id: Uuid,
    position_seconds: u64,
) -> Result<(), String> {
    let playback_service: std::sync::Arc<PlaybackService> = state.playback_service.clone();

    playback_service
        .seek_to(episode_id, position_seconds)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_playback(
    state: State<'_, AppState>,
    episode_id: Uuid,
) -> Result<(), String> {
    let playback_service: std::sync::Arc<PlaybackService> = state.playback_service.clone();

    playback_service
        .stop_playback(episode_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_episode_progress(
    state: State<'_, AppState>,
    episode_id: Uuid,
) -> Result<u64, String> {
    let playback_service: std::sync::Arc<PlaybackService> = state.playback_service.clone();

    playback_service
        .get_current_position(episode_id)
        .map_err(|e| e.to_string())
}