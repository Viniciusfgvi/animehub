use crate::domain::models::episode::{Episode, EpisodeState};
use crate::repositories::episode_repository::EpisodeRepository;
use crate::shared::errors::AppResult;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Row};
use std::str::FromStr;
use uuid::Uuid;

pub struct SqliteEpisodeRepository {
    pool: Pool<SqliteConnectionManager>,
}

impl SqliteEpisodeRepository {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    fn row_to_episode(row: &Row) -> rusqlite::Result<Episode> {
        let id_str: String = row.get(0)?;
        let anime_id_str: String = row.get(1)?;
        let state_str: String = row.get(4)?;

        Ok(Episode {
            id: Uuid::parse_str(&id_str).unwrap_or_default(),
            anime_id: Uuid::parse_str(&anime_id_str).unwrap_or_default(),
            number: row.get(2)?,
            title: row.get(3)?,
            state: EpisodeState::from_str(&state_str).unwrap_or(EpisodeState::Unwatched),
            duration: row.get(5)?,
            last_watched_at: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    }
}

impl EpisodeRepository for SqliteEpisodeRepository {
    fn save(&self, episode: &Episode) -> AppResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR REPLACE INTO episodes (id, anime_id, number, title, state, duration, last_watched_at, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                episode.id.to_string(),
                episode.anime_id.to_string(),
                episode.number,
                episode.title,
                episode.state.to_string(),
                episode.duration,
                episode.last_watched_at,
                episode.created_at,
                episode.updated_at
            ],
        )?;
        Ok(())
    }

    fn get_by_id(&self, id: Uuid) -> AppResult<Option<Episode>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, anime_id, number, title, state, duration, last_watched_at, created_at, updated_at FROM episodes WHERE id = ?1")?;
        let mut rows = stmt.query(params![id.to_string()])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_episode(row)?))
        } else {
            Ok(None)
        }
    }

    fn list_by_anime(&self, anime_id: Uuid) -> AppResult<Vec<Episode>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, anime_id, number, title, state, duration, last_watched_at, created_at, updated_at FROM episodes WHERE anime_id = ?1 ORDER BY number ASC")?;
        let rows = stmt.query_map(params![anime_id.to_string()], |row| Self::row_to_episode(row))?;

        let mut episodes = Vec::new();
        for ep in rows {
            episodes.push(ep?);
        }
        Ok(episodes)
    }

    fn list_by_state(&self, anime_id: Uuid, state: EpisodeState) -> AppResult<Vec<Episode>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, anime_id, number, title, state, duration, last_watched_at, created_at, updated_at FROM episodes WHERE anime_id = ?1 AND state = ?2")?;
        let rows = stmt.query_map(params![anime_id.to_string(), state.to_string()], |row| Self::row_to_episode(row))?;

        let mut episodes = Vec::new();
        for ep in rows {
            episodes.push(ep?);
        }
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
            "SELECT COUNT(*) FROM episodes WHERE anime_id = ?1 AND state = 'Completed'",
            params![anime_id.to_string()],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    fn link_file(&self, episode_id: Uuid, file_id: Uuid, is_primary: bool) -> AppResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR REPLACE INTO episode_files (episode_id, file_id, is_primary) VALUES (?1, ?2, ?3)",
            params![episode_id.to_string(), file_id.to_string(), is_primary],
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
        for link in rows {
            links.push(link?);
        }
        Ok(links)
    }
}