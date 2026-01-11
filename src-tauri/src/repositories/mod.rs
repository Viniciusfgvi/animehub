// src-tauri/src/repositories/mod.rs
//
// Repository layer
//
// CRITICAL RULES:
// - Repositories are DUMB data mappers
// - NO business logic
// - NO invariant enforcement
// - NO event emission
// - NO cross-repository calls
// - Explicit SQL only

// ============================================================================
// EXISTING REPOSITORY MODULES (SEALED - Phase 3)
// ============================================================================

pub mod anime_alias_repository;
pub mod anime_repository;
pub mod collection_repository;
pub mod episode_repository;
pub mod external_reference_repository;
pub mod file_repository;
pub mod statistics_repository;
pub mod subtitle_repository;

// ============================================================================
// MATERIALIZATION REPOSITORY (NEW - Phase 5)
// ============================================================================

pub mod materialization_repository;
pub mod sqlite_materialization_repository;

// ============================================================================
// PUBLIC EXPORTS - Repository Traits and SQLite Implementations
// ============================================================================

// Anime
pub use anime_repository::AnimeRepository;
pub use anime_repository::SqliteAnimeRepository;

// Episode
pub use episode_repository::EpisodeRepository;
pub use episode_repository::SqliteEpisodeRepository;

// File
pub use file_repository::FileRepository;
pub use file_repository::SqliteFileRepository;

// Subtitle
pub use subtitle_repository::SqliteSubtitleRepository;
pub use subtitle_repository::SubtitleRepository;

// Collection
pub use collection_repository::CollectionRepository;
pub use collection_repository::SqliteCollectionRepository;

// External Reference
pub use external_reference_repository::ExternalReferenceRepository;
pub use external_reference_repository::SqliteExternalReferenceRepository;

// Anime Alias
pub use anime_alias_repository::AnimeAliasRepository;
pub use anime_alias_repository::SqliteAnimeAliasRepository;

// Statistics
pub use statistics_repository::SqliteStatisticsRepository;
pub use statistics_repository::StatisticsRepository;

// Materialization (Phase 5)
pub use materialization_repository::MaterializationRepository;
