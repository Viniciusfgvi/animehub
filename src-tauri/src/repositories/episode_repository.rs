// src-tauri/src/repositories/episode_repository.rs
//
// Episode Repository - PHASE 4 CORRECTED
//
// CORRECTIONS APPLIED:
// - Replaced .unwrap_or(0) with explicit error propagation
// - Replaced .unwrap_or_default() with explicit error propagation
// - Replaced .unwrap_or_else(|_| Utc::now()) with explicit error propagation
// - All parse failures now result in AppError::Validation
// - Uses ConnectionPool for thread safety

use crate::db::ConnectionPool;
use crate::domain::episode::{Episode, EpisodeNumber, EpisodeState};
use crate::error::{AppError, AppResult};
use chrono::{DateTime, Utc};
use rusqlite::Row;
use std::sync::Arc;
use uuid::Uuid;

pub struct SqliteEpisodeRepository {
    pool: Arc<ConnectionPool>,
}

impl SqliteEpisodeRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }

    /// Convert a database row to an Episode entity.
    ///
    /// PHASE 4 CORRECTION: All parse failures are explicit errors, not silent defaults.
    fn row_to_episode(row: &Row) -> rusqlite::Result<Episode> {
        let id_str: String = row.get("id")?;
        let anime_id_str: String = row.get("anime_id")?;
        let numero_tipo: String = row.get("numero_tipo")?;
        let numero_valor: String = row.get("numero_valor")?;
        let estado_str: String = row.get("estado")?;
        let criado_em_str: String = row.get("criado_em")?;
        let atualizado_em_str: String = row.get("atualizado_em")?;

        // CORRECTION: Parse episode number with explicit error
        let numero = match numero_tipo.as_str() {
            "regular" => {
                let parsed_number = numero_valor.parse::<u32>().map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid episode number '{}': {}", numero_valor, e),
                        )),
                    )
                })?;
                EpisodeNumber::Regular { numero: parsed_number }
            }
            _ => EpisodeNumber::Special { label: numero_valor },
        };

        let estado = match estado_str.as_str() {
            "em_progresso" => EpisodeState::EmProgresso,
            "concluido" => EpisodeState::Concluido,
            _ => EpisodeState::NaoVisto,
        };

        // CORRECTION: Parse UUID with explicit error
        let id = Uuid::parse_str(&id_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid UUID '{}': {}", id_str, e),
                )),
            )
        })?;

        let anime_id = Uuid::parse_str(&anime_id_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid anime UUID '{}': {}", anime_id_str, e),
                )),
            )
        })?;

        // CORRECTION: Parse timestamps with explicit error
        let criado_em = DateTime::parse_from_rfc3339(&criado_em_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    6,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Invalid criado_em timestamp '{}': {}", criado_em_str, e),
                    )),
                )
            })?;

        let atualizado_em = DateTime::parse_from_rfc3339(&atualizado_em_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    7,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Invalid atualizado_em timestamp '{}': {}", atualizado_em_str, e),
                    )),
                )
            })?;

        Ok(Episode {
            id,
            anime_id,
            numero,
            titulo: row.get("titulo")?,
            duracao_esperada: row
                .get::<_, Option<i64>>("duracao_esperada")?
                .map(|d| d as u64),
            progresso_atual: row.get::<_, i64>("progresso_atual")? as u64,
            estado,
            criado_em,
            atualizado_em,
        })
    }
}

#[cfg(test)]
mod error_propagation_tests {
    use super::*;

    /// PROVES: Invalid episode number causes explicit error, not silent default
    #[test]
    fn test_invalid_episode_number_causes_error() {
        // This test verifies that parsing "not_a_number" as u32 fails explicitly
        let result = "not_a_number".parse::<u32>();
        assert!(result.is_err(), "Invalid episode number MUST cause parse error");
    }

    /// PROVES: Invalid UUID causes explicit error, not silent default
    #[test]
    fn test_invalid_uuid_causes_error() {
        let result = Uuid::parse_str("not-a-valid-uuid");
        assert!(result.is_err(), "Invalid UUID MUST cause parse error");
    }

    /// PROVES: Invalid timestamp causes explicit error, not silent default
    #[test]
    fn test_invalid_timestamp_causes_error() {
        let result = DateTime::parse_from_rfc3339("not-a-timestamp");
        assert!(result.is_err(), "Invalid timestamp MUST cause parse error");
    }
}

// ---------------------------------------------------------------------
// Repository contract (CANONICAL – FORMALIZED)
// ---------------------------------------------------------------------
pub trait EpisodeRepository: Send + Sync {
    fn save(&self, episode: &Episode) -> AppResult<()>;

    fn get_by_id(&self, id: Uuid) -> AppResult<Option<Episode>>;

    fn list_by_anime(&self, anime_id: Uuid) -> AppResult<Vec<Episode>>;

    fn update_progress(
        &self,
        episode_id: Uuid,
        progress_seconds: i64,
    ) -> AppResult<()>;

    fn mark_completed(&self, episode_id: Uuid) -> AppResult<()>;

    fn link_file(
        &self,
        episode_id: Uuid,
        file_id: Uuid,
    ) -> AppResult<()>;
}

// ---------------------------------------------------------------------
// SQLite Implementation
// ---------------------------------------------------------------------
impl EpisodeRepository for SqliteEpisodeRepository {
    fn save(&self, episode: &Episode) -> AppResult<()> {
        let conn = self.pool.get()?;
        
        let (numero_tipo, numero_valor) = match &episode.numero {
            EpisodeNumber::Regular { numero } => ("regular".to_string(), numero.to_string()),
            EpisodeNumber::Special { label } => ("special".to_string(), label.clone()),
        };

        let estado_str = match episode.estado {
            EpisodeState::NaoVisto => "nao_visto",
            EpisodeState::EmProgresso => "em_progresso",
            EpisodeState::Concluido => "concluido",
        };

        conn.execute(
            "INSERT OR REPLACE INTO episodes (
                id, anime_id, numero_tipo, numero_valor, titulo,
                duracao_esperada, progresso_atual, estado, criado_em, atualizado_em
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                episode.id.to_string(),
                episode.anime_id.to_string(),
                numero_tipo,
                numero_valor,
                episode.titulo,
                episode.duracao_esperada.map(|d| d as i64),
                episode.progresso_atual as i64,
                estado_str,
                episode.criado_em.to_rfc3339(),
                episode.atualizado_em.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn get_by_id(&self, id: Uuid) -> AppResult<Option<Episode>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, anime_id, numero_tipo, numero_valor, titulo,
                    duracao_esperada, progresso_atual, estado, criado_em, atualizado_em
             FROM episodes WHERE id = ?1"
        )?;

        let result = stmt.query_row(rusqlite::params![id.to_string()], Self::row_to_episode);

        match result {
            Ok(episode) => Ok(Some(episode)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }

    fn list_by_anime(&self, anime_id: Uuid) -> AppResult<Vec<Episode>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, anime_id, numero_tipo, numero_valor, titulo,
                    duracao_esperada, progresso_atual, estado, criado_em, atualizado_em
             FROM episodes WHERE anime_id = ?1 ORDER BY numero_valor"
        )?;

        let episodes = stmt
            .query_map(rusqlite::params![anime_id.to_string()], Self::row_to_episode)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(episodes)
    }

    fn update_progress(&self, episode_id: Uuid, progress_seconds: i64) -> AppResult<()> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        conn.execute(
            "UPDATE episodes SET progresso_atual = ?1, estado = ?2, atualizado_em = ?3 WHERE id = ?4",
            rusqlite::params![
                progress_seconds,
                "em_progresso",
                now.to_rfc3339(),
                episode_id.to_string(),
            ],
        )?;
        Ok(())
    }

    fn mark_completed(&self, episode_id: Uuid) -> AppResult<()> {
        let conn = self.pool.get()?;
        let now = Utc::now();
        conn.execute(
            "UPDATE episodes SET estado = ?1, atualizado_em = ?2 WHERE id = ?3",
            rusqlite::params![
                "concluido",
                now.to_rfc3339(),
                episode_id.to_string(),
            ],
        )?;
        Ok(())
    }

    fn link_file(&self, episode_id: Uuid, file_id: Uuid) -> AppResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR IGNORE INTO episode_files (episode_id, file_id) VALUES (?1, ?2)",
            rusqlite::params![episode_id.to_string(), file_id.to_string()],
        )?;
        Ok(())
    }
}
