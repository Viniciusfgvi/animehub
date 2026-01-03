// src-tauri/src/application/commands/anime_commands.rs
//
// Anime Command Handlers
//
// RULES:
// - Accept DTOs
// - Call sealed services
// - Return DTOs
// - Never contain business logic

use tauri::State;
use uuid::Uuid;
use chrono::DateTime;

use crate::application::{
    state::AppState,
    dto::*,
};
use crate::services::*;
use crate::domain::{AnimeType, AnimeStatus};

/// List all animes
#[tauri::command]
pub async fn list_animes(state: State<'_, AppState>) -> Result<Vec<AnimeDto>, String> {
    let animes = state.anime_service
        .list_all_animes()
        .map_err(|e| e.to_string())?;
    
    Ok(animes.into_iter().map(AnimeDto::from).collect())
}

/// Get a single anime by ID
#[tauri::command]
pub async fn get_anime(
    anime_id: String,
    state: State<'_, AppState>
) -> Result<Option<AnimeDto>, String> {
    let id = Uuid::parse_str(&anime_id)
        .map_err(|e| format!("Invalid UUID: {}", e))?;
    
    let anime = state.anime_service
        .get_anime(id)
        .map_err(|e| e.to_string())?;
    
    Ok(anime.map(AnimeDto::from))
}

/// Create a new anime
#[tauri::command]
pub async fn create_anime(
    dto: CreateAnimeDto,
    state: State<'_, AppState>
) -> Result<String, String> {
    // Parse type
    let tipo = match dto.tipo.as_str() {
        "TV" => AnimeType::TV,
        "Movie" => AnimeType::Movie,
        "OVA" => AnimeType::OVA,
        "Special" => AnimeType::Special,
        _ => return Err("Invalid anime type".to_string()),
    };
    
    // Parse status
    let status = match dto.status.as_str() {
        "em_exibicao" => AnimeStatus::EmExibicao,
        "finalizado" => AnimeStatus::Finalizado,
        "cancelado" => AnimeStatus::Cancelado,
        _ => return Err("Invalid status".to_string()),
    };
    
    // Parse dates
    let data_inicio = dto.data_inicio
        .map(|s| DateTime::parse_from_rfc3339(&s))
        .transpose()
        .map_err(|e| format!("Invalid start date: {}", e))?
        .map(|dt| dt.with_timezone(&chrono::Utc));
    
    let data_fim = dto.data_fim
        .map(|s| DateTime::parse_from_rfc3339(&s))
        .transpose()
        .map_err(|e| format!("Invalid end date: {}", e))?
        .map(|dt| dt.with_timezone(&chrono::Utc));
    
    // Create request
    let request = CreateAnimeRequest {
        titulo_principal: dto.titulo_principal,
        titulos_alternativos: dto.titulos_alternativos,
        tipo,
        status,
        total_episodios: dto.total_episodios,
        data_inicio,
        data_fim,
        metadados_livres: dto.metadados_livres.unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
    };
    
    let anime_id = state.anime_service
        .create_anime(request)
        .map_err(|e| e.to_string())?;
    
    Ok(anime_id.to_string())
}

/// Update an anime's metadata
#[tauri::command]
pub async fn update_anime(
    anime_id: String,
    titulo_principal: Option<String>,
    status: Option<String>,
    state: State<'_, AppState>
) -> Result<(), String> {
    let id = Uuid::parse_str(&anime_id)
        .map_err(|e| format!("Invalid UUID: {}", e))?;
    
    let parsed_status = status
        .map(|s| match s.as_str() {
            "em_exibicao" => Ok(AnimeStatus::EmExibicao),
            "finalizado" => Ok(AnimeStatus::Finalizado),
            "cancelado" => Ok(AnimeStatus::Cancelado),
            _ => Err("Invalid status".to_string()),
        })
        .transpose()?;
    
    let request = UpdateAnimeRequest {
        anime_id: id,
        titulo_principal,
        titulos_alternativos: None,
        tipo: None,
        status: parsed_status,
        total_episodios: None,
        data_inicio: None,
        data_fim: None,
        metadados_livres: None,
    };
    
    state.anime_service
        .update_anime(request)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}