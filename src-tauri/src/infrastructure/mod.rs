// src-tauri/src/infrastructure/mod.rs
//
// Infrastructure Layer
//
// Contains implementation details that support the domain
// but are not part of the domain itself.
//
// RULES:
// - Infrastructure serves the domain
// - Infrastructure never dictates domain behavior
// - Infrastructure is replaceable

pub mod subtitle_workspace;

pub use subtitle_workspace::{
    SubtitleWorkspace, SubtitleWorkspaceCleaned, SubtitleWorkspaceCreated,
};
