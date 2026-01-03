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
    ConnectionPool,
    PooledConn,
    create_connection_pool,
    get_connection,
    get_database_path,
};

pub use migrations::{
    initialize_database,
    verify_database_integrity,
    get_database_stats,
    DatabaseStats,
};