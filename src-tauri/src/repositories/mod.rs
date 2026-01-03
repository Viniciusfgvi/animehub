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

pub mod anime_repository;
pub mod episode_repository;
pub mod file_repository;
pub mod subtitle_repository;
pub mod collection_repository;
pub mod external_reference_repository;
pub mod anime_alias_repository;
pub mod statistics_repository;

pub use anime_repository::{AnimeRepository, SqliteAnimeRepository};
pub use episode_repository::{EpisodeRepository, SqliteEpisodeRepository};
pub use file_repository::{FileRepository, SqliteFileRepository};
pub use subtitle_repository::{SubtitleRepository, SqliteSubtitleRepository};
pub use collection_repository::{CollectionRepository, SqliteCollectionRepository};
pub use external_reference_repository::{ExternalReferenceRepository, SqliteExternalReferenceRepository};
pub use anime_alias_repository::{AnimeAliasRepository, SqliteAnimeAliasRepository};
pub use statistics_repository::{StatisticsRepository, SqliteStatisticsRepository};
