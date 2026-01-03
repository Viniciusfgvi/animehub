// src-tauri/src/repositories/anime_repository.rs
//
// Anime persistence - Fixed error conversion

use std::sync::Arc;
use rusqlite::{params, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::db::ConnectionPool;
use crate::domain::anime::{Anime, AnimeType, AnimeStatus};
use crate::error::{AppError, AppResult};

pub trait AnimeRepository: Send + Sync {
    fn save(&self, anime: &Anime) -> AppResult<()>;
    fn get_by_id(&self, id: Uuid) -> AppResult<Option<Anime>>;
    fn list_all(&self) -> AppResult<Vec<Anime>>;
    fn list_by_status(&self, status: AnimeStatus) -> AppResult<Vec<Anime>>;
    fn list_by_type(&self, tipo: AnimeType) -> AppResult<Vec<Anime>>;
    fn delete(&self, id: Uuid) -> AppResult<()>;
    fn exists(&self, id: Uuid) -> AppResult<bool>;
}

pub struct SqliteAnimeRepository {
    pool: Arc<ConnectionPool>,
}

impl SqliteAnimeRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }
    
    /// Map database row to Anime - returns rusqlite::Error for query_map compatibility
    fn row_to_anime(row: &Row) -> Result<Anime, rusqlite::Error> {
        let id_str: String = row.get("id")?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        
        let titulo_principal: String = row.get("titulo_principal")?;
        
        let titulos_alt_json: String = row.get("titulos_alternativos")?;
        let titulos_alternativos: Vec<String> = serde_json::from_str(&titulos_alt_json)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        
        let tipo_str: String = row.get("tipo")?;
        let tipo = match tipo_str.as_str() {
            "TV" => AnimeType::TV,
            "Movie" => AnimeType::Movie,
            "OVA" => AnimeType::OVA,
            "Special" => AnimeType::Special,
            _ => return Err(rusqlite::Error::InvalidQuery),
        };
        
        let status_str: String = row.get("status")?;
        let status = match status_str.as_str() {
            "em_exibicao" => AnimeStatus::EmExibicao,
            "finalizado" => AnimeStatus::Finalizado,
            "cancelado" => AnimeStatus::Cancelado,
            _ => return Err(rusqlite::Error::InvalidQuery),
        };
        
        let total_episodios: Option<i64> = row.get("total_episodios")?;
        let total_episodios = total_episodios.map(|v| v as u32);
        
        let data_inicio_str: Option<String> = row.get("data_inicio")?;
        let data_inicio = data_inicio_str
            .map(|s| DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e))))
            .transpose()?;
        
        let data_fim_str: Option<String> = row.get("data_fim")?;
        let data_fim = data_fim_str
            .map(|s| DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e))))
            .transpose()?;
        
        let metadados_json: String = row.get("metadados_livres")?;
        let metadados_livres: serde_json::Value = serde_json::from_str(&metadados_json)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        
        let criado_em_str: String = row.get("criado_em")?;
        let criado_em = DateTime::parse_from_rfc3339(&criado_em_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        
        let atualizado_em_str: String = row.get("atualizado_em")?;
        let atualizado_em = DateTime::parse_from_rfc3339(&atualizado_em_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        
        Ok(Anime {
            id,
            titulo_principal,
            titulos_alternativos,
            tipo,
            status,
            total_episodios,
            data_inicio,
            data_fim,
            metadados_livres,
            criado_em,
            atualizado_em,
        })
    }
}

impl AnimeRepository for SqliteAnimeRepository {
    fn save(&self, anime: &Anime) -> AppResult<()> {
        let conn = self.pool.get()?;
        
        let titulos_alt_json = serde_json::to_string(&anime.titulos_alternativos)?;
        let metadados_json = serde_json::to_string(&anime.metadados_livres)?;
        
        conn.execute(
            "INSERT OR REPLACE INTO anime (
                id, titulo_principal, titulos_alternativos, tipo, status,
                total_episodios, data_inicio, data_fim, metadados_livres,
                criado_em, atualizado_em
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                anime.id.to_string(),
                anime.titulo_principal,
                titulos_alt_json,
                anime.tipo.to_string(),
                anime.status.to_string(),
                anime.total_episodios.map(|v| v as i64),
                anime.data_inicio.map(|dt| dt.to_rfc3339()),
                anime.data_fim.map(|dt| dt.to_rfc3339()),
                metadados_json,
                anime.criado_em.to_rfc3339(),
                anime.atualizado_em.to_rfc3339(),
            ]
        )?;
        
        Ok(())
    }
    
    fn get_by_id(&self, id: Uuid) -> AppResult<Option<Anime>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare(
            "SELECT id, titulo_principal, titulos_alternativos, tipo, status,
                    total_episodios, data_inicio, data_fim, metadados_livres,
                    criado_em, atualizado_em
             FROM anime WHERE id = ?1"
        )?;
        
        match stmt.query_row(params![id.to_string()], Self::row_to_anime) {
            Ok(anime) => Ok(Some(anime)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }
    
    fn list_all(&self) -> AppResult<Vec<Anime>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare(
            "SELECT id, titulo_principal, titulos_alternativos, tipo, status,
                    total_episodios, data_inicio, data_fim, metadados_livres,
                    criado_em, atualizado_em
             FROM anime
             ORDER BY titulo_principal"
        )?;
        
        let animes: Vec<Anime> = stmt.query_map([], Self::row_to_anime)?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(animes)
    }
    
    fn list_by_status(&self, status: AnimeStatus) -> AppResult<Vec<Anime>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare(
            "SELECT id, titulo_principal, titulos_alternativos, tipo, status,
                    total_episodios, data_inicio, data_fim, metadados_livres,
                    criado_em, atualizado_em
             FROM anime
             WHERE status = ?1
             ORDER BY titulo_principal"
        )?;
        
        let animes: Vec<Anime> = stmt.query_map(params![status.to_string()], Self::row_to_anime)?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(animes)
    }
    
    fn list_by_type(&self, tipo: AnimeType) -> AppResult<Vec<Anime>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare(
            "SELECT id, titulo_principal, titulos_alternativos, tipo, status,
                    total_episodios, data_inicio, data_fim, metadados_livres,
                    criado_em, atualizado_em
             FROM anime
             WHERE tipo = ?1
             ORDER BY titulo_principal"
        )?;
        
        let animes: Vec<Anime> = stmt.query_map(params![tipo.to_string()], Self::row_to_anime)?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(animes)
    }
    
    fn delete(&self, id: Uuid) -> AppResult<()> {
    let conn = self.pool.get()?;
    
    let rows_affected = conn.execute(
        "DELETE FROM anime WHERE id = ?1",
        params![id.to_string()]
    )?;
    
    if rows_affected == 0 {
        // ✅ FIXED: Changed from AppError::not_found(...) to AppError::NotFound
        return Err(AppError::NotFound);
    }
    
    Ok(())
    }
    
    fn exists(&self, id: Uuid) -> AppResult<bool> {
        let conn = self.pool.get()?;
        
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM anime WHERE id = ?1",
            params![id.to_string()],
            |row| row.get(0)
        )?;
        
        Ok(count > 0)
    }
}