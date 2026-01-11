// src-tauri/src/db/mod.rs
//
// Database module
//
// Provides:
// - Connection pooling
// - Schema migrations
// - Database utilities

pub mod connection;
pub mod migrations;

pub use connection::{
    create_connection_pool, get_connection, get_database_path, ConnectionPool, PooledConn,
};

pub use migrations::{
    get_database_stats, initialize_database, verify_database_integrity, DatabaseStats,
};
