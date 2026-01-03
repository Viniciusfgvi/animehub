// src-tauri/src/db/connection.rs
//
// Database connection management
//
// PRINCIPLES:
// - Explicit connection pooling
// - No hidden connection creation
// - Clear error propagation
// - Thread-safe access

use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::PathBuf;

use crate::error::{AppError, AppResult};

/// Type alias for connection pool
pub type ConnectionPool = Pool<SqliteConnectionManager>;

/// Type alias for a pooled connection
pub type PooledConn = PooledConnection<SqliteConnectionManager>;

/// Get the database file path
/// 
/// Database is stored in the application data directory.
/// Path structure: {APP_DATA}/animehub/animehub.db
pub fn get_database_path() -> AppResult<PathBuf> {
    let app_data_dir = dirs::data_dir()
        .ok_or_else(|| AppError::Other("Could not determine app data directory".to_string()))?;
    
    let animehub_dir = app_data_dir.join("animehub");
    
    // Ensure directory exists
    std::fs::create_dir_all(&animehub_dir)
        .map_err(|e| AppError::Io(e))?;
    
    Ok(animehub_dir.join("animehub.db"))
}

/// Create a connection pool
/// 
/// Pool configuration:
/// - Max 15 connections (reasonable for desktop app)
/// - SQLite in WAL mode for better concurrency
/// - Foreign keys enabled
/// - Busy timeout set to avoid immediate errors
pub fn create_connection_pool() -> AppResult<ConnectionPool> {
    let db_path = get_database_path()?;
    
    let manager = SqliteConnectionManager::file(&db_path)
        .with_init(|conn| {
            // Enable foreign key support (not default in SQLite)
            conn.execute_batch(
                "PRAGMA foreign_keys = ON;
                 PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA busy_timeout = 5000;"
            )?;
            Ok(())
        });
    
    let pool = Pool::builder()
        .max_size(15)
        .build(manager)
        .map_err(|e| AppError::Other(format!("Failed to create connection pool: {}", e)))?;
    
    Ok(pool)
}

/// Get a connection from the pool
/// 
/// This is a convenience wrapper that provides better error messages.
pub fn get_connection(pool: &ConnectionPool) -> AppResult<PooledConn> {
    pool.get()
        .map_err(|e| AppError::Other(format!("Failed to get database connection: {}", e)))
}

/// Create a standalone connection (for testing)
/// 
/// This creates an in-memory database, useful for unit tests.
pub fn create_test_connection() -> AppResult<Connection> {
    let conn = Connection::open_in_memory()
        .map_err(AppError::Database)?;
    
    // Enable foreign keys
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;"
    ).map_err(AppError::Database)?;
    
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_database_path_creation() {
        let path = get_database_path().unwrap();
        assert!(path.ends_with("animehub/animehub.db"));
    }
    
    #[test]
    fn test_connection_pool_creation() {
        // This will create actual database file in app data
        let pool = create_connection_pool().unwrap();
        let conn = get_connection(&pool).unwrap();
        
        // Verify foreign keys are enabled
        let fk_enabled: i32 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk_enabled, 1);
    }
    
    #[test]
    fn test_test_connection() {
        let conn = create_test_connection().unwrap();
        
        // Verify it's a working connection
        let result: i32 = conn
            .query_row("SELECT 1 + 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(result, 2);
        
        // Verify foreign keys are enabled
        let fk_enabled: i32 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk_enabled, 1);
    }
}