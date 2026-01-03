// src-tauri/src/repositories/statistics_repository.rs

use std::sync::Arc;
use rusqlite::{params, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::db::ConnectionPool;
use crate::domain::statistics::{StatisticsSnapshot, StatisticsType};
use crate::error::{AppError, AppResult};

pub trait StatisticsRepository: Send + Sync {
    fn save_snapshot(&self, snapshot: &StatisticsSnapshot) -> AppResult<()>;
    fn get_snapshot_by_type(&self, tipo: &str) -> AppResult<Option<StatisticsSnapshot>>;
    fn list_all_snapshots(&self) -> AppResult<Vec<StatisticsSnapshot>>;
    fn delete_all(&self) -> AppResult<()>;
}

pub struct SqliteStatisticsRepository {
    pool: Arc<ConnectionPool>,
}

impl SqliteStatisticsRepository {
    pub fn new(pool: Arc<ConnectionPool>) -> Self {
        Self { pool }
    }

    fn row_to_snapshot(row: &Row) -> Result<StatisticsSnapshot, rusqlite::Error> {
        let id_str: String = row.get("id")?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let tipo_raw: String = row.get("tipo")?;
        let tipo = if tipo_raw == "global" {
            StatisticsType::Global
        } else if let Some(anime_id_str) = tipo_raw.strip_prefix("por_anime:") {
            let anime_id = Uuid::parse_str(anime_id_str)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            StatisticsType::PorAnime { anime_id }
        } else if let Some(period_str) = tipo_raw.strip_prefix("por_periodo:") {
            let parts: Vec<&str> = period_str.split(':').collect();
            if parts.len() == 2 {
                let inicio = DateTime::parse_from_rfc3339(parts[0])
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
                    .with_timezone(&Utc);
                let fim = DateTime::parse_from_rfc3339(parts[1])
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
                    .with_timezone(&Utc);
                StatisticsType::PorPeriodo { inicio, fim }
            } else {
                return Err(rusqlite::Error::InvalidQuery);
            }
        } else {
            return Err(rusqlite::Error::InvalidQuery);
        };

        let valor_json: String = row.get("valor")?;
        let valor = serde_json::from_str(&valor_json)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let gerado_em = DateTime::parse_from_rfc3339(&row.get::<_, String>("gerado_em")?)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
            .with_timezone(&Utc);

        Ok(StatisticsSnapshot {
            id,
            tipo,
            valor,
            gerado_em,
        })
    }
}

impl StatisticsRepository for SqliteStatisticsRepository {
    fn save_snapshot(&self, snap: &StatisticsSnapshot) -> AppResult<()> {
        let conn = self.pool.get()?;
        let valor_json = serde_json::to_string(&snap.valor)?;
        
        conn.execute(
            "INSERT OR REPLACE INTO statistics_snapshots (id, tipo, valor, gerado_em)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                snap.id.to_string(),
                snap.tipo.to_string(),
                valor_json,
                snap.gerado_em.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    fn get_snapshot_by_type(&self, tipo: &str) -> AppResult<Option<StatisticsSnapshot>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare(
            "SELECT * FROM statistics_snapshots WHERE tipo = ?1 ORDER BY gerado_em DESC LIMIT 1"
        )?;
        
        match stmt.query_row(params![tipo], Self::row_to_snapshot) {
            Ok(snap) => Ok(Some(snap)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e)),
        }
    }

    fn list_all_snapshots(&self) -> AppResult<Vec<StatisticsSnapshot>> {
        let conn = self.pool.get()?;
        
        let mut stmt = conn.prepare(
            "SELECT * FROM statistics_snapshots ORDER BY gerado_em DESC"
        )?;
        
        let snapshots: Vec<StatisticsSnapshot> = stmt.query_map([], Self::row_to_snapshot)?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(snapshots)
    }

    fn delete_all(&self) -> AppResult<()> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM statistics_snapshots", [])?;
        Ok(())
    }
}