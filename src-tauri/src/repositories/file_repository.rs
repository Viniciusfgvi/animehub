use crate::domain::file::{File, FileType, FileOrigin};
use crate::error::AppResult;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Row};
use std::sync::Arc;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub trait FileRepository: Send + Sync {
    fn save(&self, file: &File) -> AppResult<()>;
    fn get_by_id(&self, id: Uuid) -> AppResult<Option<File>>;
    fn get_by_path(&self, path: &PathBuf) -> AppResult<Option<File>>;
    fn list_by_type(&self, file_type: FileType) -> AppResult<Vec<File>>;
    fn delete(&self, id: Uuid) -> AppResult<()>;
    fn exists(&self, id: Uuid) -> AppResult<bool>;
}

pub struct SqliteFileRepository {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl SqliteFileRepository {
    pub fn new(pool: Arc<Pool<SqliteConnectionManager>>) -> Self {
        Self { pool }
    }

    fn row_to_file(row: &Row) -> rusqlite::Result<File> {
        let id_str: String = row.get("id")?;
        let path_str: String = row.get("caminho_absoluto")?;
        let tipo_str: String = row.get("tipo")?;
        let origem_str: String = row.get("origem")?;
        
        let tipo = match tipo_str.as_str() {
            "video" => FileType::Video,
            "legenda" => FileType::Legenda,
            "imagem" => FileType::Imagem,
            _ => FileType::Outro,
        };

        let origem = match origem_str.as_str() {
            "scan" => FileOrigin::Scan,
            "importacao" => FileOrigin::Importacao,
            _ => FileOrigin::Manual,
        };

        Ok(File {
            id: Uuid::parse_str(&id_str).unwrap_or_default(),
            caminho_absoluto: PathBuf::from(path_str),
            tipo,
            tamanho: row.get::<_, i64>("tamanho")? as u64,
            hash: row.get("hash")?,
            data_modificacao: DateTime::parse_from_rfc3339(&row.get::<_, String>("data_modificacao")?)
                .map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
            origem,
            criado_em: DateTime::parse_from_rfc3339(&row.get::<_, String>("criado_em")?)
                .map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
            atualizado_em: DateTime::parse_from_rfc3339(&row.get::<_, String>("atualizado_em")?)
                .map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
        })
    }
}

impl FileRepository for SqliteFileRepository {
    fn save(&self, file: &File) -> AppResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR REPLACE INTO files (id, caminho_absoluto, tipo, tamanho, hash, data_modificacao, origem, criado_em, atualizado_em)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                file.id.to_string(),
                file.caminho_absoluto.to_string_lossy().to_string(),
                file.tipo.to_string(),
                file.tamanho as i64,
                file.hash,
                file.data_modificacao.to_rfc3339(),
                file.origem.to_string(),
                file.criado_em.to_rfc3339(),
                file.atualizado_em.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    fn get_by_id(&self, id: Uuid) -> AppResult<Option<File>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM files WHERE id = ?1")?;
        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_file(row)?))
        } else {
            Ok(None)
        }
    }

    fn get_by_path(&self, path: &PathBuf) -> AppResult<Option<File>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM files WHERE caminho_absoluto = ?1")?;
        let mut rows = stmt.query(params![path.to_string_lossy().to_string()])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_file(row)?))
        } else {
            Ok(None)
        }
    }

    fn list_by_type(&self, file_type: FileType) -> AppResult<Vec<File>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM files WHERE tipo = ?1")?;
        let files = stmt.query_map(params![file_type.to_string()], Self::row_to_file)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(files)
    }

    fn delete(&self, id: Uuid) -> AppResult<()> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM files WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    fn exists(&self, id: Uuid) -> AppResult<bool> {
        let conn = self.pool.get()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM files WHERE id = ?1",
            params![id.to_string()],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}