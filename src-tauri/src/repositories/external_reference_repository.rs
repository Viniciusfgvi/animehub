// src-tauri/src/repositories/external_reference_repository.rs

use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use std::sync::Arc;
use uuid::Uuid;

use crate::db::ConnectionPool;
use crate::domain::ExternalReference;
use crate::error::{AppError, AppResult};

pub trait ExternalReferenceRepository: Send + Sync {
    fn save(&self, reference: &ExternalReference) -> AppResult<()>;
    fn get_by_anime_and_source(
        &self,
        anime_id: Uuid,
        fonte: &str,
    ) -> AppResult<Option<ExternalReference>>;
    fn list_by_anime(&self, anime_id: Uuid) -> AppResult<Vec<ExternalReference>>;
    fn delete(&self, id: Uuid) -> AppResult<()>;
}

pub struct SqliteExternalReferenceRepository {
    pool: Arc<ConnectionPool>,
}

impl SqliteExternalReferenceRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }

    fn row_to_reference(row: &Row) -> Result<ExternalReference, rusqlite::Error> {
        let id = Uuid::parse_str(&row.get::<_, String>("id")?)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let anime_id = Uuid::parse_str(&row.get::<_, String>("anime_id")?)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let fonte: String = row.get("fonte")?;
        let external_id: String = row.get("external_id")?;

        let criado_em = DateTime::parse_from_rfc3339(&row.get::<_, String>("criado_em")?)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
            .with_timezone(&Utc);

        Ok(ExternalReference {
            id,
            anime_id,
            fonte,
            external_id,
            criado_em,
        })
    }
}

impl ExternalReferenceRepository for SqliteExternalReferenceRepository {
    fn save(&self, reference: &ExternalReference) -> AppResult<()> {
        let conn = self.pool.get()?;

        conn.execute(
            "INSERT OR REPLACE INTO external_references (id, anime_id, fonte, external_id, criado_em)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                reference.id.to_string(),
                reference.anime_id.to_string(),
                reference.fonte,
                reference.external_id,
                reference.criado_em.to_rfc3339(),
            ]
        )?;

        Ok(())
    }

    fn get_by_anime_and_source(
        &self,
        anime_id: Uuid,
        fonte: &str,
    ) -> AppResult<Option<ExternalReference>> {
        let conn = self.pool.get()?;

        let mut stmt =
            conn.prepare("SELECT * FROM external_references WHERE anime_id = ?1 AND fonte = ?2")?;

        match stmt.query_row(params![anime_id.to_string(), fonte], Self::row_to_reference) {
            Ok(reference) => Ok(Some(reference)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }

    fn list_by_anime(&self, anime_id: Uuid) -> AppResult<Vec<ExternalReference>> {
        let conn = self.pool.get()?;

        let mut stmt = conn.prepare("SELECT * FROM external_references WHERE anime_id = ?1")?;

        let refs: Vec<ExternalReference> = stmt
            .query_map(params![anime_id.to_string()], Self::row_to_reference)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(refs)
    }

    fn delete(&self, id: Uuid) -> AppResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "DELETE FROM external_references WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }
}
