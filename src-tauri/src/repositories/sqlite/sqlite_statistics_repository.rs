use crate::domain::models::statistics::{StatisticsSnapshot, StatisticsType};
use crate::repositories::statistics_repository::StatisticsRepository;
use crate::shared::errors::AppResult;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Row};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

pub struct SqliteStatisticsRepository {
    pool: Pool<SqliteConnectionManager>,
}

impl SqliteStatisticsRepository {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    fn row_to_snapshot(row: &Row) -> rusqlite::Result<StatisticsSnapshot> {
        let id_str: String = row.get(0)?;
        let tipo_str: String = row.get(1)?;
        let data_json: String = row.get(2)?;

        // Agora utiliza o FromStr do domínio, removendo o "cheiro" arquitetural
        let tipo = StatisticsType::from_str(&tipo_str)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let data: HashMap<String, f64> = serde_json::from_str(&data_json)
            .unwrap_or_default();

        Ok(StatisticsSnapshot {
            id: Uuid::parse_str(&id_str).unwrap_or_default(),
            tipo,
            data,
            captured_at: row.get(3)?,
        })
    }
}

impl StatisticsRepository for SqliteStatisticsRepository {
    fn save_snapshot(&self, snapshot: &StatisticsSnapshot) -> AppResult<()> {
        let conn = self.pool.get()?;
        let data_json = serde_json::to_string(&snapshot.data).unwrap_or_default();
        
        conn.execute(
            "INSERT OR REPLACE INTO statistics_snapshots (id, tipo, data, captured_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                snapshot.id.to_string(),
                snapshot.tipo.to_string(),
                data_json,
                snapshot.captured_at
            ],
        )?;
        Ok(())
    }

    fn get_snapshot_by_type(&self, tipo: &str) -> AppResult<Option<StatisticsSnapshot>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, tipo, data, captured_at FROM statistics_snapshots WHERE tipo = ?1")?;
        let mut rows = stmt.query(params![tipo])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_snapshot(row)?))
        } else {
            Ok(None)
        }
    }

    fn list_all_snapshots(&self) -> AppResult<Vec<StatisticsSnapshot>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, tipo, data, captured_at FROM statistics_snapshots ORDER BY captured_at DESC")?;
        let rows = stmt.query_map([], |row| Self::row_to_snapshot(row))?;

        let mut snapshots = Vec::new();
        for s in rows {
            snapshots.push(s?);
        }
        Ok(snapshots)
    }

    fn delete_all(&self) -> AppResult<()> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM statistics_snapshots", [])?;
        Ok(())
    }
}