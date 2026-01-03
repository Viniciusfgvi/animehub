// src-tauri/src/application/dto/mod.rs
//
// Data Transfer Objects
//
// CRITICAL PRINCIPLES:
// - DTOs are UI-friendly representations
// - DTOs NEVER leak domain invariants
// - DTOs are simple, serializable structs
// - Conversion FROM domain entities only (never TO)

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// ANIME DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeDto {
    pub id: String,
    pub titulo_principal: String,
    pub titulos_alternativos: Vec<String>,
    pub tipo: String,
    pub status: String,
    pub total_episodios: Option<u32>,
    pub data_inicio: Option<String>,
    pub data_fim: Option<String>,
    pub metadados_livres: serde_json::Value,
    pub criado_em: String,
    pub atualizado_em: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAnimeDto {
    pub titulo_principal: String,
    pub titulos_alternativos: Vec<String>,
    pub tipo: String,
    pub status: String,
    pub total_episodios: Option<u32>,
    pub data_inicio: Option<String>,
    pub data_fim: Option<String>,
    pub metadados_livres: Option<serde_json::Value>,
}

// ============================================================================
// EPISODE DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeDto {
    pub id: String,
    pub anime_id: String,
    pub numero: String,
    pub titulo: Option<String>,
    pub duracao_esperada: Option<u64>,
    pub progresso_atual: u64,
    pub estado: String,
    pub criado_em: String,
    pub atualizado_em: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEpisodeDto {
    pub anime_id: String,
    pub numero: String,
    pub numero_tipo: String, // "regular" or "special"
    pub titulo: Option<String>,
    pub duracao_esperada: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgressDto {
    pub episode_id: String,
    pub progress_seconds: u64,
}

// ============================================================================
// FILE DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDto {
    pub id: String,
    pub caminho_absoluto: String,
    pub tipo: String,
    pub tamanho: u64,
    pub hash: Option<String>,
    pub data_modificacao: String,
    pub origem: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanDirectoryDto {
    pub directory_path: String,
}

// ============================================================================
// PLAYBACK DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartPlaybackDto {
    pub episode_id: String,
    pub file_id: Option<String>,
    pub start_position: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackStatusDto {
    pub episode_id: String,
    pub current_position: u64,
    pub is_playing: bool,
}

// ============================================================================
// STATISTICS DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalStatisticsDto {
    pub total_animes: u32,
    pub total_episodes: u32,
    pub episodes_assistidos: u32,
    pub tempo_total_assistido: u64,
    pub animes_em_progresso: u32,
    pub animes_completos: u32,
}

// ============================================================================
// RESPONSE DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
}

impl<T> SuccessResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            success: true,
            data,
        }
    }
}

impl ErrorResponse {
    pub fn new(error: String) -> Self {
        Self {
            success: false,
            error,
        }
    }
}

// ============================================================================
// CONVERSION HELPERS (Domain → DTO)
// ============================================================================

impl From<crate::domain::Anime> for AnimeDto {
    fn from(anime: crate::domain::Anime) -> Self {
        Self {
            id: anime.id.to_string(),
            titulo_principal: anime.titulo_principal,
            titulos_alternativos: anime.titulos_alternativos,
            tipo: anime.tipo.to_string(),
            status: anime.status.to_string(),
            total_episodios: anime.total_episodios,
            data_inicio: anime.data_inicio.map(|d| d.to_rfc3339()),
            data_fim: anime.data_fim.map(|d| d.to_rfc3339()),
            metadados_livres: anime.metadados_livres,
            criado_em: anime.criado_em.to_rfc3339(),
            atualizado_em: anime.atualizado_em.to_rfc3339(),
        }
    }
}

impl From<crate::domain::Episode> for EpisodeDto {
    fn from(episode: crate::domain::Episode) -> Self {
        Self {
            id: episode.id.to_string(),
            anime_id: episode.anime_id.to_string(),
            numero: episode.numero.to_string(),
            titulo: episode.titulo,
            duracao_esperada: episode.duracao_esperada,
            progresso_atual: episode.progresso_atual,
            estado: episode.estado.to_string(),
            criado_em: episode.criado_em.to_rfc3339(),
            atualizado_em: episode.atualizado_em.to_rfc3339(),
        }
    }
}

impl From<crate::domain::File> for FileDto {
    fn from(file: crate::domain::File) -> Self {
        Self {
            id: file.id.to_string(),
            caminho_absoluto: file.caminho_absoluto.to_string_lossy().to_string(),
            tipo: file.tipo.to_string(),
            tamanho: file.tamanho,
            hash: file.hash,
            data_modificacao: file.data_modificacao.to_rfc3339(),
            origem: file.origem.to_string(),
        }
    }
}

impl From<crate::domain::GlobalStatistics> for GlobalStatisticsDto {
    fn from(stats: crate::domain::GlobalStatistics) -> Self {
        Self {
            total_animes: stats.total_animes,
            total_episodes: stats.total_episodes,
            episodes_assistidos: stats.episodes_assistidos,
            tempo_total_assistido: stats.tempo_total_assistido,
            animes_em_progresso: stats.animes_em_progresso,
            animes_completos: stats.animes_completos,
        }
    }
}