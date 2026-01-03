// src-tauri/src/repositories/anime_alias_repository.rs

use std::sync::Arc;
use rusqlite::params;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::db::ConnectionPool;
use crate::domain::AnimeAlias;
use crate::error::{AppError, AppResult};

pub trait AnimeAliasRepository: Send + Sync {
    fn save(&self, alias: &AnimeAlias) -> AppResult<()>;
    fn get_principal_for_alias(&self, anime_alias_id: Uuid) -> AppResult<Option<Uuid>>;
    fn list_aliases_for_principal(&self, anime_principal_id: Uuid) -> AppResult<Vec<AnimeAlias>>;
}

pub struct SqliteAnimeAliasRepository {
    pool: Arc<ConnectionPool>,
}

impl SqliteAnimeAliasRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }
}

impl AnimeAliasRepository for SqliteAnimeAliasRepository {
    fn save(&self, alias: &AnimeAlias) -> AppResult<()> {
        let conn = self.pool.get()?;
        
        conn.execute(
            "INSERT OR REPLACE INTO anime_aliases (id, anime_principal_id, anime_alias_id, criado_em)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                alias.id.to_string(),
                alias.anime_principal_id.to_string(),
                alias.anime_alias_id.to_string(),
                alias.criado_em.to_rfc3339(),
            ]
        )?;
        
        Ok(())
    }
    
    fn get_principal_for_alias(&self, anime_alias_id: Uuid) -> AppResult<Option<Uuid>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare(
            "SELECT anime_principal_id FROM anime_aliases WHERE anime_alias_id = ?1"
        )?;
        
        match stmt.query_row(params![anime_alias_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            Ok(id_str)
        }) {
            Ok(id_str) => Ok(Some(Uuid::parse_str(&id_str)?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }
    
    fn list_aliases_for_principal(&self, anime_principal_id: Uuid) -> AppResult<Vec<AnimeAlias>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare(
            "SELECT id, anime_principal_id, anime_alias_id, criado_em 
             FROM anime_aliases 
             WHERE anime_principal_id = ?1"
        )?;
        
        let aliases: Vec<AnimeAlias> = stmt.query_map(
            params![anime_principal_id.to_string()],
            |row| {
                let id = Uuid::parse_str(&row.get::<_, String>(0)?)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                let principal_id = Uuid::parse_str(&row.get::<_, String>(1)?)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                let alias_id = Uuid::parse_str(&row.get::<_, String>(2)?)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                let criado_em = DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
                    .with_timezone(&Utc);
                
                Ok(AnimeAlias {
                    id,
                    anime_principal_id: principal_id,
                    anime_alias_id: alias_id,
                    criado_em,
                })
            }
        )?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(aliases)
    }
}