// src-tauri/src/application/commands/mod.rs
//
// Tauri Command Handlers
//
// ARCHITECTURE:
// - Commands are thin adapters between UI and Services
// - Commands accept DTOs, return DTOs
// - Commands handle error conversion for Tauri
// - Commands NEVER contain business logic

pub mod anime_commands;
pub mod episode_commands;
pub mod file_commands;
pub mod playback_commands;
pub mod statistics_commands;

pub use anime_commands::*;
pub use episode_commands::*;
pub use file_commands::*;
pub use playback_commands::*;
pub use statistics_commands::*;
