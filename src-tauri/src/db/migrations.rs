// src-tauri/src/db/migrations.rs
//
// Database schema initialization and migrations
//
// PRINCIPLES:
// - Explicit schema versions
// - No automatic migrations
// - Clear error messages
// - Idempotent operations

use rusqlite::Connection;
use crate::error::{AppError, AppResult};

/// Current schema version
/// Increment this when adding migrations
const CURRENT_SCHEMA_VERSION: i32 = 1;

/// Initialize the database schema
/// 
/// This function:
/// 1. Checks current schema version
/// 2. Applies necessary migrations
/// 3. Updates version tracking
/// 
/// Safe to call multiple times (idempotent).
pub fn initialize_database(conn: &Connection) -> AppResult<()> {
    let current_version = get_schema_version(conn)?;
    
    if current_version == 0 {
        // Fresh database - apply initial schema
        apply_initial_schema(conn)?;
        set_schema_version(conn, 1)?;
    } else if current_version < CURRENT_SCHEMA_VERSION {
        // Future: apply incremental migrations here
        // For now, we only have version 1
        return Err(AppError::Other(
            format!("Schema version {} is outdated. Expected {}. Manual migration required.",
                current_version, CURRENT_SCHEMA_VERSION)
        ));
    } else if current_version > CURRENT_SCHEMA_VERSION {
        return Err(AppError::Other(
            format!("Schema version {} is newer than supported {}. Update the application.",
                current_version, CURRENT_SCHEMA_VERSION)
        ));
    }
    
    Ok(())
}

/// Get current schema version
/// Returns 0 if schema_version table doesn't exist (fresh database)
fn get_schema_version(conn: &Connection) -> AppResult<i32> {
    // Check if schema_version table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='schema_version')",
            [],
            |row| row.get(0)
        )
        .map_err(AppError::Database)?;
    
    if !table_exists {
        return Ok(0);
    }
    
    // Get the highest version number
    let version: Option<i32> = conn
        .query_row(
            "SELECT MAX(version) FROM schema_version",
            [],
            |row| row.get(0)
        )
        .map_err(AppError::Database)?;
    
    Ok(version.unwrap_or(0))
}

/// Set schema version
fn set_schema_version(conn: &Connection, version: i32) -> AppResult<()> {
    conn.execute(
        "INSERT OR IGNORE INTO schema_version (version, applied_at) VALUES (?1, datetime('now'))",
        [version]
    ).map_err(AppError::Database)?;
    
    Ok(())
}

/// Apply initial schema (version 1)
/// 
/// This includes all tables defined in schema.sql
fn apply_initial_schema(conn: &Connection) -> AppResult<()> {
    // Read schema from embedded file
    let schema = include_str!("../../schema.sql");
    
    // Execute as batch
    conn.execute_batch(schema)
        .map_err(|e| AppError::Other(format!("Failed to apply initial schema: {}", e)))?;
    
    Ok(())
}

/// Verify database integrity
/// 
/// Runs SQLite's integrity check. Should be called periodically.
pub fn verify_database_integrity(conn: &Connection) -> AppResult<()> {
    let result: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(AppError::Database)?;
    
    if result != "ok" {
        return Err(AppError::Other(format!("Database integrity check failed: {}", result)));
    }
    
    Ok(())
}

/// Get database statistics
/// 
/// Returns useful info for debugging and monitoring
pub fn get_database_stats(conn: &Connection) -> AppResult<DatabaseStats> {
    let page_count: i64 = conn
        .query_row("PRAGMA page_count", [], |row| row.get(0))
        .map_err(AppError::Database)?;
    
    let page_size: i64 = conn
        .query_row("PRAGMA page_size", [], |row| row.get(0))
        .map_err(AppError::Database)?;
    
    let size_bytes = page_count * page_size;
    
    // Get row counts for main tables
    let anime_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM anime", [], |row| row.get(0))
        .unwrap_or(0);
    
    let episode_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM episodes", [], |row| row.get(0))
        .unwrap_or(0);
    
    let file_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
        .unwrap_or(0);
    
    Ok(DatabaseStats {
        size_bytes,
        page_count,
        page_size,
        anime_count,
        episode_count,
        file_count,
    })
}

/// Database statistics
#[derive(Debug)]
pub struct DatabaseStats {
    pub size_bytes: i64,
    pub page_count: i64,
    pub page_size: i64,
    pub anime_count: i64,
    pub episode_count: i64,
    pub file_count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::create_test_connection;
    
    #[test]
    fn test_initialize_fresh_database() {
        let conn = create_test_connection().unwrap();
        
        // Should be version 0 initially
        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, 0);
        
        // Initialize
        initialize_database(&conn).unwrap();
        
        // Should now be version 1
        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, 1);
        
        // Verify tables exist
        let table_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
                [],
                |row| row.get(0)
            )
            .unwrap();
        
        assert!(table_count > 10, "Expected at least 10 tables, got {}", table_count);
    }
    
    #[test]
    fn test_initialize_idempotent() {
        let conn = create_test_connection().unwrap();
        
        // Initialize twice
        initialize_database(&conn).unwrap();
        initialize_database(&conn).unwrap();
        
        // Should still be version 1
        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, 1);
    }
    
    #[test]
    fn test_foreign_keys_enabled() {
        let conn = create_test_connection().unwrap();
        initialize_database(&conn).unwrap();
        
        // Try to insert episode without anime (should fail)
        let result = conn.execute(
            "INSERT INTO episodes (id, anime_id, numero_tipo, numero_valor, progresso_atual, estado, criado_em, atualizado_em)
             VALUES ('test-ep', 'nonexistent-anime', 'regular', '1', 0, 'nao_visto', datetime('now'), datetime('now'))",
            []
        );
        
        assert!(result.is_err(), "Foreign key constraint should have been violated");
    }
    
    #[test]
    fn test_database_stats() {
        let conn = create_test_connection().unwrap();
        initialize_database(&conn).unwrap();
        
        let stats = get_database_stats(&conn).unwrap();
        
        assert!(stats.size_bytes > 0);
        assert_eq!(stats.anime_count, 0);
        assert_eq!(stats.episode_count, 0);
        assert_eq!(stats.file_count, 0);
    }
    
    #[test]
    fn test_integrity_check() {
        let conn = create_test_connection().unwrap();
        initialize_database(&conn).unwrap();
        
        // Fresh database should pass integrity check
        verify_database_integrity(&conn).unwrap();
    }
}