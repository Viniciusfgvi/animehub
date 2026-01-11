// src-tauri/src/lib.rs
// AnimeHub - Local-first anime library manager
//
// Architecture:
// - Domain-centric: All business logic lives in domains (SEALED - Phase 3)
// - Event-driven: Services coordinate through events (SEALED - Phase 3)
// - Explicit: No implicit behavior, no magic
// - Local-first: User controls all data
// - Application Layer: UI boundary (Phase 4)
// - Materialization: Domain entity creation from resolution (Phase 5)

// ============================================================================
// SEALED FOUNDATION (Phase 3)
// ============================================================================

pub mod db;
pub mod domain;
pub mod error;
pub mod events;
pub mod infrastructure;
pub mod repositories;
pub mod services;

// ============================================================================
// APPLICATION LAYER (Phase 4)
// ============================================================================

pub mod application;
pub mod integrations;

// ============================================================================
// PUBLIC API - Domain Entities (Sealed)
// ============================================================================

pub use domain::{
    validate_anime,
    validate_anime_alias,
    validate_collection,
    validate_episode,
    validate_external_reference,
    validate_file,
    validate_subtitle,
    // Anime
    Anime,
    // Anime Alias
    AnimeAlias,
    AnimeStatistics,
    AnimeStatus,
    AnimeType,
    // Collection
    Collection,
    // Episode
    Episode,
    EpisodeNumber,
    EpisodeState,
    // External Reference
    ExternalReference,
    // File
    File,
    FileOrigin,
    FileType,
    GlobalStatistics,
    // Statistics
    StatisticsSnapshot,
    StatisticsType,
    // Subtitle
    Subtitle,
    SubtitleFormat,
    SubtitleTransformation,
    TransformationType,
};

// ============================================================================
// PUBLIC API - Error Types (Sealed)
// ============================================================================

pub use error::{AppError, AppResult};

// ============================================================================
// PUBLIC API - Events (Sealed)
// ============================================================================

pub use events::{
    create_event_bus,
    register_materialization_handlers,
    // Common events
    AnimeCreated,
    DomainEvent,
    EpisodeCompleted,
    EpisodeCreated,
    EpisodeResolved,
    EventBus,
    EventLogEntry,
    FileLinkedToEpisode,
    // Resolution events (Phase 4)
    FileResolved,
    // Materialization events (Phase 5)
    MaterializationBatchCompleted,
    MaterializationRecordCreated,
    PlaybackStarted,
    ResolutionBatchCompleted,
    ResolutionFailed,
};

// ============================================================================
// PUBLIC API - Database (Sealed)
// ============================================================================

pub use db::{create_connection_pool, initialize_database, ConnectionPool};

// ============================================================================
// PUBLIC API - Repositories (Sealed)
// ============================================================================

pub use repositories::{
    AnimeAliasRepository,
    AnimeRepository,
    CollectionRepository,
    EpisodeRepository,
    ExternalReferenceRepository,
    FileRepository,
    // Materialization (Phase 5)
    MaterializationRepository,
    SqliteAnimeAliasRepository,
    SqliteAnimeRepository,
    SqliteCollectionRepository,
    SqliteExternalReferenceRepository,
    SqliteStatisticsRepository,
    SqliteSubtitleRepository,
    StatisticsRepository,
    SubtitleRepository,
};

// ============================================================================
// PUBLIC API - Infrastructure (Sealed)
// ============================================================================

pub use infrastructure::{SubtitleWorkspace, SubtitleWorkspaceCleaned, SubtitleWorkspaceCreated};

// ============================================================================
// PUBLIC API - Services (Sealed)
// ============================================================================

pub use services::{
    // Anime Service
    AnimeService,
    CreateAnimeRequest,
    CreateEpisodeRequest,
    EpisodeNumberDecision,
    // Episode Service
    EpisodeService,
    // External Integration Service
    ExternalIntegrationService,
    ExternalMetadata,
    FetchMetadataRequest,
    // File Service
    FileService,
    LinkExternalReferenceRequest,
    LinkFileRequest,

    MaterializationDecision,
    MaterializationEventType,
    MaterializationFingerprint,
    MaterializationOutcome,
    MaterializationRecord,
    MaterializationResult,
    // Materialization Service (Phase 5)
    MaterializationService,
    MergeAnimesRequest,

    MetadataSuggestions,

    ObserverConfig,

    // Playback Observer
    PlaybackObserver,
    // Playback Service
    PlaybackService,
    RegisterFileRequest,

    ResolutionRules,

    // Resolution Service (Phase 4 - FROZEN)
    ResolutionService,
    StartPlaybackRequest,

    // Statistics Service
    StatisticsService,

    StyleTransformRequest,
    // Subtitle Service
    SubtitleService,
    TimingTransformRequest,

    UpdateAnimeRequest,
    UpdateEpisodeMetadataRequest,
};

// ============================================================================
// PUBLIC API - Application Layer (Phase 4)
// ============================================================================

pub use application::AppState;

// Re-export application submodules
pub use application::commands;
pub use application::dto;

// ============================================================================
// PUBLIC API - Integrations (Phase 4 - Stubs)
// ============================================================================

pub use integrations::{AniListAnime, AniListClient, MpvClient};
