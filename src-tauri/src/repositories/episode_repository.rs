// src-tauri/src/repositories/episode_repository.rs
//
// Episode Repository - Data Mapper for Episode Entity
//
// CRITICAL RULES:
// - Repositories are DUMB data mappers
// - NO business logic
// - NO invariant enforcement
// - NO event emission
// - NO cross-repository calls
// - Explicit SQL only

use std::sync::Arc;
use rusqlite::{params, Row};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::episode::{Episode, EpisodeNumber, EpisodeState};
use crate::error::AppResult;

pub trait EpisodeRepository: Send + Sync {
    fn save(&self, episode: &Episode) -> AppResult<()>;
    fn get_by_id(&self, id: Uuid) -> AppResult<Option<Episode>>;
    fn list_by_anime(&self, anime_id: Uuid) -> AppResult<Vec<Episode>>;
    fn list_by_state(&self, anime_id: Uuid, state: EpisodeState) -> AppResult<Vec<Episode>>;
    fn delete(&self, id: Uuid) -> AppResult<()>;
    fn exists(&self, id: Uuid) -> AppResult<bool>;
    fn count_by_anime(&self, anime_id: Uuid) -> AppResult<usize>;
    fn count_completed(&self, anime_id: Uuid) -> AppResult<usize>;
    fn link_file(&self, episode_id: Uuid, file_id: Uuid, is_primary: bool) -> AppResult<()>;
    fn unlink_file(&self, episode_id: Uuid, file_id: Uuid) -> AppResult<()>;
    fn get_linked_files(&self, episode_id: Uuid) -> AppResult<Vec<(Uuid, bool)>>;
    
    // ========================================================================
    // READ-ONLY FIND METHODS (Added for Phase 4 Resolution)
    // ========================================================================
    
    /// Find an episode by anime ID and regular episode number.
    /// Returns None if no matching episode exists.
    /// This is a READ-ONLY operation - no state mutation.
    fn find_by_anime_and_number(&self, anime_id: Uuid, number: u32) -> AppResult<Option<Episode>>;
    
    /// Find an episode by anime ID and special label (e.g., "OVA", "Special 1").
    /// Returns None if no matching episode exists.
    /// This is a READ-ONLY operation - no state mutation.
    fn find_by_anime_and_special_label(&self, anime_id: Uuid, label: &str) -> AppResult<Option<Episode>>;
}

pub struct SqliteEpisodeRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl SqliteEpisodeRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    fn row_to_episode(row: &Row) -> rusqlite::Result<Episode> {
        let id_str: String = row.get("id")?;
        let anime_id_str: String = row.get("anime_id")?;
        let numero_tipo: String = row.get("numero_tipo")?;
        let numero_valor: String = row.get("numero_valor")?;
        let estado_str: String = row.get("estado")?;

        let numero = match numero_tipo.as_str() {
            "regular" => EpisodeNumber::Regular { 
                numero: numero_valor.parse().unwrap_or(0) 
            },
            _ => EpisodeNumber::Special { label: numero_valor },
        };

        let estado = match estado_str.as_str() {
            "em_progresso" => EpisodeState::EmProgresso,
            "concluido" => EpisodeState::Concluido,
            _ => EpisodeState::NaoVisto,
        };

        Ok(Episode {
            id: Uuid::parse_str(&id_str).unwrap_or_default(),
            anime_id: Uuid::parse_str(&anime_id_str).unwrap_or_default(),
            numero,
            titulo: row.get("titulo")?,
            duracao_esperada: row.get::<_, Option<i64>>("duracao_esperada")?.map(|d| d as u64),
            progresso_atual: row.get::<_, i64>("progresso_atual")? as u64,
            estado,
            criado_em: DateTime::parse_from_rfc3339(&row.get::<_, String>("criado_em")?)
                .map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
            atualizado_em: DateTime::parse_from_rfc3339(&row.get::<_, String>("atualizado_em")?)
                .map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
        })
    }
}

impl EpisodeRepository for SqliteEpisodeRepository {
    fn save(&self, ep: &Episode) -> AppResult<()> {
        let conn = self.pool.get()?;
        let (num_tipo, num_val) = match &ep.numero {
            EpisodeNumber::Regular { numero } => ("regular", numero.to_string()),
            EpisodeNumber::Special { label } => ("special", label.clone()),
        };

        conn.execute(
            "INSERT OR REPLACE INTO episodes (id, anime_id, numero_tipo, numero_valor, titulo, duracao_esperada, progresso_atual, estado, criado_em, atualizado_em)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                ep.id.to_string(),
                ep.anime_id.to_string(),
                num_tipo,
                num_val,
                ep.titulo,
                ep.duracao_esperada.map(|d| d as i64),
                ep.progresso_atual as i64,
                ep.estado.to_string(),
                ep.criado_em.to_rfc3339(),
                ep.atualizado_em.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    fn get_by_id(&self, id: Uuid) -> AppResult<Option<Episode>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM episodes WHERE id = ?1")?;
        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_episode(row)?))
        } else {
            Ok(None)
        }
    }

    fn list_by_anime(&self, anime_id: Uuid) -> AppResult<Vec<Episode>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM episodes WHERE anime_id = ?1 ORDER BY numero_valor ASC")?;
        let episodes = stmt.query_map(params![anime_id.to_string()], Self::row_to_episode)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(episodes)
    }

    fn list_by_state(&self, anime_id: Uuid, state: EpisodeState) -> AppResult<Vec<Episode>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM episodes WHERE anime_id = ?1 AND estado = ?2")?;
        let episodes = stmt.query_map(params![anime_id.to_string(), state.to_string()], Self::row_to_episode)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(episodes)
    }

    fn delete(&self, id: Uuid) -> AppResult<()> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM episodes WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    fn exists(&self, id: Uuid) -> AppResult<bool> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM episodes WHERE id = ?1",
            params![id.to_string()],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    fn count_by_anime(&self, anime_id: Uuid) -> AppResult<usize> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM episodes WHERE anime_id = ?1",
            params![anime_id.to_string()],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    fn count_completed(&self, anime_id: Uuid) -> AppResult<usize> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM episodes WHERE anime_id = ?1 AND estado = 'concluido'",
            params![anime_id.to_string()],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    fn link_file(&self, episode_id: Uuid, file_id: Uuid, is_primary: bool) -> AppResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR REPLACE INTO episode_files (episode_id, file_id, is_primary, criado_em) VALUES (?1, ?2, ?3, ?4)",
            params![episode_id.to_string(), file_id.to_string(), is_primary, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    fn unlink_file(&self, episode_id: Uuid, file_id: Uuid) -> AppResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "DELETE FROM episode_files WHERE episode_id = ?1 AND file_id = ?2",
            params![episode_id.to_string(), file_id.to_string()],
        )?;
        Ok(())
    }

    fn get_linked_files(&self, episode_id: Uuid) -> AppResult<Vec<(Uuid, bool)>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT file_id, is_primary FROM episode_files WHERE episode_id = ?1")?;
        let rows = stmt.query_map(params![episode_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            Ok((Uuid::parse_str(&id_str).unwrap_or_default(), row.get(1)?))
        })?;
        let mut links = Vec::new();
        for link in rows { links.push(link?); }
        Ok(links)
    }

    // ========================================================================
    // READ-ONLY FIND METHODS (Added for Phase 4 Resolution)
    // ========================================================================

    fn find_by_anime_and_number(&self, anime_id: Uuid, number: u32) -> AppResult<Option<Episode>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT * FROM episodes WHERE anime_id = ?1 AND numero_tipo = 'regular' AND numero_valor = ?2"
        )?;
        let mut rows = stmt.query(params![anime_id.to_string(), number.to_string()])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_episode(row)?))
        } else {
            Ok(None)
        }
    }

    fn find_by_anime_and_special_label(&self, anime_id: Uuid, label: &str) -> AppResult<Option<Episode>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT * FROM episodes WHERE anime_id = ?1 AND numero_tipo = 'special' AND numero_valor = ?2"
        )?;
        let mut rows = stmt.query(params![anime_id.to_string(), label])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_episode(row)?))
        } else {
            Ok(None)
        }
    }
}
