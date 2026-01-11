// src-tauri/src/app/materialization_init.rs
//
// Materialization Initialization - Phase 5
//
// Application-level initialization for the materialization subsystem.
// This file shows how to wire up the MaterializationService with the event bus.
//
// CRITICAL RULES:
// - Uses existing repository implementations
// - Uses existing event bus
// - Does not modify existing initialization code
// - Additive integration only

use std::sync::Arc;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::events::EventBus;
use crate::events::handlers::register_materialization_handlers;
use crate::repositories::anime_repository::AnimeRepository;
use crate::repositories::EpisodeRepository;

use crate::repositories::FileRepository;

use crate::repositories::materialization_repository::MaterializationRepository;
use crate::repositories::sqlite_materialization_repository::SqliteMaterializationRepository;
use crate::services::materialization_service::MaterializationService;
use crate::error::AppResult;

// ============================================================================
// MATERIALIZATION SUBSYSTEM INITIALIZATION
// ============================================================================

/// Configuration for the materialization subsystem
pub struct MaterializationConfig {
    /// Whether to enable automatic materialization on resolution events
    pub auto_materialize: bool,
}

impl Default for MaterializationConfig {
    fn default() -> Self {
        Self {
            auto_materialize: true,
        }
    }
}

/// Initializes the materialization subsystem.
/// 
/// This function:
/// 1. Creates the MaterializationRepository
/// 2. Creates the MaterializationService
/// 3. Registers event handlers with the event bus
/// 
/// # Arguments
/// * `anime_repo` - The anime repository implementation
/// * `episode_repo` - The episode repository implementation
/// * `file_repo` - The file repository implementation
/// * `pool` - The SQLite connection pool for materialization records
/// * `event_bus` - The application event bus
/// * `config` - Configuration options
/// 
/// # Returns
/// The initialized MaterializationService wrapped in Arc
pub fn init_materialization_subsystem(
    anime_repo: Arc<dyn AnimeRepository>,
    episode_repo: Arc<dyn EpisodeRepository>,
    file_repo: Arc<dyn FileRepository>,
    pool: Arc<Pool<SqliteConnectionManager>>,
    event_bus: Arc<EventBus>,
    config: MaterializationConfig,
) -> AppResult<Arc<MaterializationService>> {
    println!("[MATERIALIZATION] Initializing subsystem...");

    // Create materialization repository
    let mat_repo: Arc<dyn MaterializationRepository> = Arc::new(
        SqliteMaterializationRepository::new(pool)?
    );

    // Create materialization service
    let service = Arc::new(MaterializationService::new(
        anime_repo,
        episode_repo,
        file_repo,
        mat_repo,
        event_bus.clone(),
    ));

    // Register event handlers if auto-materialize is enabled
    if config.auto_materialize {
        register_materialization_handlers(&event_bus, service.clone());
        println!("[MATERIALIZATION] Event handlers registered");
    }

    println!("[MATERIALIZATION] Subsystem initialized");
    Ok(service)
}

// ============================================================================
// INTEGRATION EXAMPLE
// ============================================================================

/// Example of how to integrate materialization into the main application.
/// This is NOT executable code, but a reference for integration.
/// 
/// ```rust,ignore
/// // In main.rs or app initialization:
/// 
/// use animehub::app::materialization_init::{
///     init_materialization_subsystem,
///     MaterializationConfig,
/// };
/// 
/// // Assuming these are already created:
/// // - anime_repo: Arc<dyn AnimeRepository>
/// // - episode_repo: Arc<dyn EpisodeRepository>
/// // - file_repo: Arc<dyn FileRepository>
/// // - pool: Arc<Pool<SqliteConnectionManager>>
/// // - event_bus: Arc<EventBus>
/// 
/// let mat_service = init_materialization_subsystem(
///     anime_repo,
///     episode_repo,
///     file_repo,
///     pool,
///     event_bus,
///     MaterializationConfig::default(),
/// )?;
/// 
/// // The materialization service is now active and will automatically
/// // process FileResolved and EpisodeResolved events from the event bus.
/// 
/// // For manual materialization (e.g., in tests or CLI):
/// let event = FileResolved::new(...);
/// let result = mat_service.materialize_file_resolved(&event)?;
/// ```
#[cfg(doctest)]
fn _integration_example() {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MaterializationConfig::default();
        assert!(config.auto_materialize);
    }
}
