// src-tauri/src/application/commands/episode_commands.rs

use tauri::State;
use uuid::Uuid;

use crate::application::{dto::*, state::AppState};
use crate::domain::EpisodeNumber;
use crate::services::*;

/// List all episodes for an anime
#[tauri::command]
pub async fn list_episodes(
    anime_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<EpisodeDto>, String> {
    let id = Uuid::parse_str(&anime_id).map_err(|e| format!("Invalid UUID: {}", e))?;

    let episodes = state
        .episode_service
        .list_episodes_for_anime(id)
        .map_err(|e| e.to_string())?;

    Ok(episodes.into_iter().map(EpisodeDto::from).collect())
}

/// Get a single episode by ID
#[tauri::command]
pub async fn get_episode(
    episode_id: String,
    state: State<'_, AppState>,
) -> Result<Option<EpisodeDto>, String> {
    let id = Uuid::parse_str(&episode_id).map_err(|e| format!("Invalid UUID: {}", e))?;

    let episode = state
        .episode_service
        .get_episode(id)
        .map_err(|e| e.to_string())?;

    Ok(episode.map(EpisodeDto::from))
}

/// Create a new episode
#[tauri::command]
pub async fn create_episode(
    dto: CreateEpisodeDto,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let anime_id =
        Uuid::parse_str(&dto.anime_id).map_err(|e| format!("Invalid anime UUID: {}", e))?;

    let numero = match dto.numero_tipo.as_str() {
        "regular" => {
            let num: u32 = dto
                .numero
                .parse()
                .map_err(|e| format!("Invalid episode number: {}", e))?;
            EpisodeNumber::regular(num)
        }
        "special" => EpisodeNumber::special(dto.numero.clone()),
        _ => return Err("Invalid numero_tipo, must be 'regular' or 'special'".to_string()),
    };

    let request = CreateEpisodeRequest {
        anime_id,
        numero,
        titulo: dto.titulo,
        duracao_esperada: dto.duracao_esperada,
    };

    let episode_id = state
        .episode_service
        .create_episode(request)
        .map_err(|e| e.to_string())?;

    Ok(episode_id.to_string())
}

/// Update episode progress
#[tauri::command]
pub async fn update_progress(
    dto: UpdateProgressDto,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let episode_id =
        Uuid::parse_str(&dto.episode_id).map_err(|e| format!("Invalid UUID: {}", e))?;

    state
        .episode_service
        .update_progress(episode_id, dto.progress_seconds)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Mark episode as completed
#[tauri::command]
pub async fn mark_episode_completed(
    episode_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let id = Uuid::parse_str(&episode_id).map_err(|e| format!("Invalid UUID: {}", e))?;

    state
        .episode_service
        .mark_completed(id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Reset episode progress
#[tauri::command]
pub async fn reset_episode_progress(
    episode_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let id = Uuid::parse_str(&episode_id).map_err(|e| format!("Invalid UUID: {}", e))?;

    state
        .episode_service
        .reset_progress(id)
        .map_err(|e| e.to_string())?;

    Ok(())
}
