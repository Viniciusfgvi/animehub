// src-tauri/src/repositories/collection_repository.rs

use std::sync::Arc;
use rusqlite::{params, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::db::ConnectionPool;
use crate::domain::collection::Collection;
use crate::error::{AppError, AppResult};

pub trait CollectionRepository: Send + Sync {
    fn save(&self, collection: &Collection) -> AppResult<()>;
    fn get_by_id(&self, id: Uuid) -> AppResult<Option<Collection>>;
    fn list_all(&self) -> AppResult<Vec<Collection>>;
    fn delete(&self, id: Uuid) -> AppResult<()>;
    fn add_anime(&self, collection_id: Uuid, anime_id: Uuid) -> AppResult<()>;
    fn remove_anime(&self, collection_id: Uuid, anime_id: Uuid) -> AppResult<()>;
    fn list_anime_in_collection(&self, collection_id: Uuid) -> AppResult<Vec<Uuid>>;
    fn list_collections_for_anime(&self, anime_id: Uuid) -> AppResult<Vec<Uuid>>;
}

pub struct SqliteCollectionRepository {
    pool: Arc<ConnectionPool>,
}

impl SqliteCollectionRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }
    
    fn row_to_collection(row: &Row) -> Result<Collection, rusqlite::Error> {
        let id = Uuid::parse_str(&row.get::<_, String>("id")?)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let nome: String = row.get("nome")?;
        let descricao: Option<String> = row.get("descricao")?;
        
        let criado_em = DateTime::parse_from_rfc3339(&row.get::<_, String>("criado_em")?)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
            .with_timezone(&Utc);
        
        Ok(Collection {
            id,
            nome,
            descricao,
            criado_em,
        })
    }
}

impl CollectionRepository for SqliteCollectionRepository {
    fn save(&self, collection: &Collection) -> AppResult<()> {
        let conn = self.pool.get()?;
        
        conn.execute(
            "INSERT OR REPLACE INTO collections (id, nome, descricao, criado_em)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                collection.id.to_string(),
                collection.nome,
                collection.descricao,
                collection.criado_em.to_rfc3339(),
            ]
        )?;
        
        Ok(())
    }
    
    fn get_by_id(&self, id: Uuid) -> AppResult<Option<Collection>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare("SELECT * FROM collections WHERE id = ?1")?;
        
        match stmt.query_row(params![id.to_string()], Self::row_to_collection) {
            Ok(col) => Ok(Some(col)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }
    
    fn list_all(&self) -> AppResult<Vec<Collection>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare("SELECT * FROM collections ORDER BY nome")?;
        
        let collections: Vec<Collection> = stmt.query_map([], Self::row_to_collection)?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(collections)
    }
    
    fn delete(&self, id: Uuid) -> AppResult<()> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM collections WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }
    
    fn add_anime(&self, collection_id: Uuid, anime_id: Uuid) -> AppResult<()> {
        let conn = self.pool.get()?;
        
        conn.execute(
            "INSERT OR IGNORE INTO anime_collections (anime_id, collection_id, criado_em)
             VALUES (?1, ?2, datetime('now'))",
            params![anime_id.to_string(), collection_id.to_string()]
        )?;
        
        Ok(())
    }
    
    fn remove_anime(&self, collection_id: Uuid, anime_id: Uuid) -> AppResult<()> {
        let conn = self.pool.get()?;
        
        conn.execute(
            "DELETE FROM anime_collections WHERE collection_id = ?1 AND anime_id = ?2",
            params![collection_id.to_string(), anime_id.to_string()]
        )?;
        
        Ok(())
    }
    
    fn list_anime_in_collection(&self, collection_id: Uuid) -> AppResult<Vec<Uuid>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare(
            "SELECT anime_id FROM anime_collections WHERE collection_id = ?1"
        )?;
        
        let anime_ids: Vec<Uuid> = stmt.query_map(params![collection_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            Uuid::parse_str(&id_str)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(anime_ids)
    }
    
    fn list_collections_for_anime(&self, anime_id: Uuid) -> AppResult<Vec<Uuid>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare(
            "SELECT collection_id FROM anime_collections WHERE anime_id = ?1"
        )?;
        
        let collection_ids: Vec<Uuid> = stmt.query_map(params![anime_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            Uuid::parse_str(&id_str)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(collection_ids)
    }
}