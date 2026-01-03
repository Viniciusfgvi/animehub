// src-tauri/src/integrations/mod.rs
//
// External Integrations Module
//
// Phase 4: Stub implementations
// Phase 5: Full implementation

pub mod anilist;
pub mod mpv;

pub use anilist::client::{AniListClient, AniListAnime, AniListTitle, AniListDate};
pub use mpv::client::MpvClient;