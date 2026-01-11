// src-tauri/src/repositories/subtitle_repository.rs

use chrono::{DateTime, Utc};
use rusqlite::{params, Row};
use std::sync::Arc;
use uuid::Uuid;

use crate::db::ConnectionPool;
use crate::domain::subtitle::{
    Subtitle, SubtitleFormat, SubtitleTransformation, TransformationType,
};
use crate::error::{AppError, AppResult};

pub trait SubtitleRepository: Send + Sync {
    fn save_subtitle(&self, subtitle: &Subtitle) -> AppResult<()>;
    fn get_subtitle_by_id(&self, id: Uuid) -> AppResult<Option<Subtitle>>;
    fn list_by_file(&self, file_id: Uuid) -> AppResult<Vec<Subtitle>>;
    fn list_by_language(&self, language: &str) -> AppResult<Vec<Subtitle>>;
    fn save_transformation(&self, transformation: &SubtitleTransformation) -> AppResult<()>;
    fn get_transformations(&self, subtitle_id: Uuid) -> AppResult<Vec<SubtitleTransformation>>;
}

pub struct SqliteSubtitleRepository {
    pool: Arc<ConnectionPool>,
}

impl SqliteSubtitleRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }

    fn row_to_subtitle(row: &Row) -> Result<Subtitle, rusqlite::Error> {
        let id = Uuid::parse_str(&row.get::<_, String>("id")?)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let file_id = Uuid::parse_str(&row.get::<_, String>("file_id")?)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let formato_str: String = row.get("formato")?;
        let formato = match formato_str.as_str() {
            "SRT" => SubtitleFormat::SRT,
            "ASS" => SubtitleFormat::ASS,
            "VTT" => SubtitleFormat::VTT,
            _ => return Err(rusqlite::Error::InvalidQuery),
        };

        let idioma: String = row.get("idioma")?;
        let versao: i32 = row.get("versao")?;
        let eh_original: i32 = row.get("eh_original")?;

        let criado_em = DateTime::parse_from_rfc3339(&row.get::<_, String>("criado_em")?)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
            .with_timezone(&Utc);

        Ok(Subtitle {
            id,
            file_id,
            formato,
            idioma,
            versao: versao as u32,
            eh_original: eh_original == 1,
            criado_em,
        })
    }

    fn row_to_transformation(row: &Row) -> Result<SubtitleTransformation, rusqlite::Error> {
        let id = Uuid::parse_str(&row.get::<_, String>("id")?)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let subtitle_id_origem = Uuid::parse_str(&row.get::<_, String>("subtitle_id_origem")?)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let tipo_str: String = row.get("tipo")?;
        let tipo = match tipo_str.as_str() {
            "style" => TransformationType::Style,
            "timing" => TransformationType::Timing,
            "conversao" => TransformationType::Conversao,
            _ => return Err(rusqlite::Error::InvalidQuery),
        };

        let parametros_json: String = row.get("parametros_aplicados")?;
        let parametros_aplicados: serde_json::Value = serde_json::from_str(&parametros_json)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let criado_em = DateTime::parse_from_rfc3339(&row.get::<_, String>("criado_em")?)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
            .with_timezone(&Utc);

        Ok(SubtitleTransformation {
            id,
            subtitle_id_origem,
            tipo,
            parametros_aplicados,
            criado_em,
        })
    }
}

impl SubtitleRepository for SqliteSubtitleRepository {
    fn save_subtitle(&self, subtitle: &Subtitle) -> AppResult<()> {
        let conn = self.pool.get()?;

        conn.execute(
            "INSERT OR REPLACE INTO subtitles (
                id, file_id, formato, idioma, versao, eh_original, criado_em
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                subtitle.id.to_string(),
                subtitle.file_id.to_string(),
                subtitle.formato.to_string(),
                &subtitle.idioma,
                subtitle.versao as i32,
                subtitle.eh_original as i32,
                subtitle.criado_em.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    fn get_subtitle_by_id(&self, id: Uuid) -> AppResult<Option<Subtitle>> {
        let conn = self.pool.get()?;

        let mut stmt = conn.prepare("SELECT * FROM subtitles WHERE id = ?1")?;

        match stmt.query_row(params![id.to_string()], Self::row_to_subtitle) {
            Ok(sub) => Ok(Some(sub)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }

    fn list_by_file(&self, file_id: Uuid) -> AppResult<Vec<Subtitle>> {
        let conn = self.pool.get()?;

        let mut stmt = conn.prepare("SELECT * FROM subtitles WHERE file_id = ?1")?;

        let subs: Vec<Subtitle> = stmt
            .query_map(params![file_id.to_string()], Self::row_to_subtitle)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(subs)
    }

    fn list_by_language(&self, language: &str) -> AppResult<Vec<Subtitle>> {
        let conn = self.pool.get()?;

        let mut stmt = conn.prepare("SELECT * FROM subtitles WHERE idioma = ?1")?;

        let subs: Vec<Subtitle> = stmt
            .query_map(params![language], Self::row_to_subtitle)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(subs)
    }

    fn save_transformation(&self, transformation: &SubtitleTransformation) -> AppResult<()> {
        let conn = self.pool.get()?;

        let parametros_json = serde_json::to_string(&transformation.parametros_aplicados)?;

        conn.execute(
            "INSERT INTO subtitle_transformations (
                id, subtitle_id_origem, tipo, parametros_aplicados, criado_em
            ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                transformation.id.to_string(),
                transformation.subtitle_id_origem.to_string(),
                transformation.tipo.to_string(),
                parametros_json,
                transformation.criado_em.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    fn get_transformations(&self, subtitle_id: Uuid) -> AppResult<Vec<SubtitleTransformation>> {
        let conn = self.pool.get()?;

        let mut stmt = conn.prepare(
            "SELECT * FROM subtitle_transformations WHERE subtitle_id_origem = ?1 ORDER BY criado_em"
        )?;

        let transformations: Vec<SubtitleTransformation> = stmt
            .query_map(
                params![subtitle_id.to_string()],
                Self::row_to_transformation,
            )?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transformations)
    }
}
