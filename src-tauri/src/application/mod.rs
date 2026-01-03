// src-tauri/src/application/mod.rs
//
// Application Layer - Phase 4
//
// ARCHITECTURE:
// - This layer sits ABOVE the sealed foundation
// - It provides the boundary between UI (Tauri) and Domain (Services)
// - It never modifies sealed components
// - It translates between DTOs and domain entities

pub mod dto;
pub mod commands;
pub mod state;

pub use dto::*;
pub use commands::*;
pub use state::AppState;