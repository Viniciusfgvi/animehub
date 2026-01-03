// src-tauri/src/application/commands/statistics_commands.rs

use tauri::State;

use crate::application::{
    state::AppState,
    dto::*,
};

/// Get global statistics
#[tauri::command]
pub async fn get_global_statistics(
    state: State<'_, AppState>
) -> Result<GlobalStatisticsDto, String> {
    let stats = state.statistics_service
        .calculate_global_statistics()
        .map_err(|e| e.to_string())?;
    
    Ok(GlobalStatisticsDto::from(stats))
}