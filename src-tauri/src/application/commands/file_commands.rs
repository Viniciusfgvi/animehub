// src-tauri/src/application/commands/file_commands.rs

use tauri::State;
use std::path::PathBuf;

use crate::application::{
    state::AppState,
    dto::*,
};

/// Scan a directory for video and subtitle files
#[tauri::command]
pub async fn scan_directory(
    dto: ScanDirectoryDto,
    state: State<'_, AppState>
) -> Result<usize, String> {
    let path = PathBuf::from(dto.directory_path);
    
    let files_found = state.file_service
        .scan_directory(path)
        .map_err(|e| e.to_string())?;
    
    Ok(files_found)
}

/// Get linked files for an episode
#[tauri::command]
pub async fn get_episode_files(
    episode_id: String,
    state: State<'_, AppState>
) -> Result<Vec<String>, String> {
    let id = uuid::Uuid::parse_str(&episode_id)
        .map_err(|e| format!("Invalid UUID: {}", e))?;
    
    let linked_files = state.episode_service
        .get_linked_files(id)
        .map_err(|e| e.to_string())?;
    
    Ok(linked_files.into_iter().map(|(file_id, _)| file_id.to_string()).collect())
}