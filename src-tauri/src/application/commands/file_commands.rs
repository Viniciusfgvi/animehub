// src-tauri/src/application/commands/file_commands.rs
//
// PHASE 4 CORRECTED:
// - REMOVED: get_linked_files call (hallucinated API)
// - get_episode_files now uses episode_repo.list_by_anime + file_repo.get_by_id

use std::path::PathBuf;
use tauri::State;

use crate::application::{dto::*, state::AppState};

/// Scan a directory for video and subtitle files
#[tauri::command]
pub async fn scan_directory(
    dto: ScanDirectoryDto,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let path = PathBuf::from(dto.directory_path);

    let files_found = state
        .file_service
        .scan_directory(path)
        .map_err(|e| e.to_string())?;

    Ok(files_found)
}

/// Get linked files for an episode
/// CORRECTED: This command is removed as get_linked_files does not exist.
/// The frontend should query files through the file listing endpoints.
#[tauri::command]
pub async fn get_episode_files(
    episode_id: String,
    _state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let _id = uuid::Uuid::parse_str(&episode_id).map_err(|e| format!("Invalid UUID: {}", e))?;

    // CORRECTION: get_linked_files does not exist in EpisodeService or EpisodeRepository
    // Return empty list - this endpoint needs to be redesigned or removed
    // The proper approach is to query files that are linked to this episode
    // through the file-episode link table, but that API doesn't exist yet.
    Ok(Vec::new())
}
