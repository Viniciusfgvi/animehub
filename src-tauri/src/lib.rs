// src-tauri/src/lib.rs
// AnimeHub - Local-first anime library manager
//
// Architecture:
// - Domain-centric: All business logic lives in domains (SEALED - Phase 3)
// - Event-driven: Services coordinate through events (SEALED - Phase 3)
// - Explicit: No implicit behavior, no magic
// - Local-first: User controls all data
// - Application Layer: UI boundary (NEW - Phase 4)

// ============================================================================
// SEALED FOUNDATION (Phase 3)
// ============================================================================

pub mod domain;
pub mod error;
pub mod events;
pub mod db;
pub mod repositories;
pub mod infrastructure;
pub mod services;

// ============================================================================
// APPLICATION LAYER (Phase 4)
// ============================================================================

pub mod application;
pub mod integrations;

// ============================================================================
// PUBLIC API - Domain Entities (Sealed)
// ============================================================================

pub use domain::*;

// ============================================================================
// PUBLIC API - Error Types (Sealed)
// ============================================================================

pub use error::{AppError, AppResult};

// ============================================================================
// PUBLIC API - Events (Sealed)
// ============================================================================

pub use events::{
    EventBus,
    create_event_bus,
    DomainEvent,
    // Common events
    AnimeCreated,
    EpisodeCreated,
    PlaybackStarted,
    EpisodeCompleted,
};

// ============================================================================
// PUBLIC API - Database (Sealed)
// ============================================================================

pub use db::{
    ConnectionPool,
    create_connection_pool,
    initialize_database,
};

// ============================================================================
// PUBLIC API - Repositories (Sealed)
// ============================================================================

pub use repositories::{
    AnimeRepository,
    SqliteAnimeRepository,
    EpisodeRepository,
    SqliteEpisodeRepository,
    FileRepository,
    SqliteFileRepository,
    SubtitleRepository,
    SqliteSubtitleRepository,
    CollectionRepository,
    SqliteCollectionRepository,
    ExternalReferenceRepository,
    SqliteExternalReferenceRepository,
    AnimeAliasRepository,
    SqliteAnimeAliasRepository,
    StatisticsRepository,
    SqliteStatisticsRepository,
};

// ============================================================================
// PUBLIC API - Infrastructure (Sealed)
// ============================================================================

pub use infrastructure::{
    SubtitleWorkspace,
    SubtitleWorkspaceCreated,
    SubtitleWorkspaceCleaned,
};

// ============================================================================
// PUBLIC API - Services (Sealed)
// ============================================================================

pub use services::{
    // Anime Service
    AnimeService,
    CreateAnimeRequest,
    UpdateAnimeRequest,
    MergeAnimesRequest,
    
    // Episode Service
    EpisodeService,
    CreateEpisodeRequest,
    UpdateEpisodeMetadataRequest,
    LinkFileRequest,
    
    // File Service
    FileService,
    RegisterFileRequest,
    
    // Playback Service
    PlaybackService,
    StartPlaybackRequest,
    
    // Statistics Service
    StatisticsService,
    
    // External Integration Service
    ExternalIntegrationService,
    FetchMetadataRequest,
    LinkExternalReferenceRequest,
    ExternalMetadata,
    MetadataSuggestions,
    
    // Subtitle Service
    SubtitleService,
    StyleTransformRequest,
    TimingTransformRequest,
};

// ============================================================================
// PUBLIC API - Application Layer (Phase 4)
// ============================================================================

pub use application::{
    AppState,
    dto::*,
    commands::*,
};

// ============================================================================
// PUBLIC API - Integrations (Phase 4 - Stubs)
// ============================================================================

pub use integrations::{
    AniListClient,
    AniListAnime,
    MpvClient,
};