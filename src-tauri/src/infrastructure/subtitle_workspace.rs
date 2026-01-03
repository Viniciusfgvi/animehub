// src-tauri/src/infrastructure/subtitle_workspace.rs
//
// Subtitle Workspace Management
//
// CRITICAL RULES:
// - One workspace per subtitle transformation session
// - Original files are NEVER modified
// - Cleanup requires explicit confirmation
// - All operations are traceable

use std::path::{Path, PathBuf};
use std::fs;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::error::{AppError, AppResult};

/// Represents a temporary workspace for subtitle transformations
/// 
/// INVARIANTS:
/// - Each workspace has a unique ID
/// - Workspace contains a working copy of the subtitle
/// - Original file is never touched
/// - Cleanup must be explicit
#[derive(Debug, Clone)]
pub struct SubtitleWorkspace {
    /// Unique workspace identifier
    pub id: Uuid,
    
    /// Path to the workspace directory
    pub workspace_dir: PathBuf,
    
    /// Path to the original subtitle file (read-only)
    pub original_file: PathBuf,
    
    /// Path to the working copy in workspace
    pub working_file: PathBuf,
    
    /// When this workspace was created
    pub created_at: DateTime<Utc>,
    
    /// Whether this workspace has been cleaned up
    pub is_cleaned: bool,
}

impl SubtitleWorkspace {
    /// Create a new workspace for a subtitle file
    /// 
    /// This will:
    /// 1. Create a temporary directory
    /// 2. Copy the original file into it
    /// 3. Return a workspace handle
    /// 
    /// The original file is NEVER modified.
    pub fn new(original_file: PathBuf) -> AppResult<Self> {
        // Validate original file exists
        if !original_file.exists() {
            return Err(AppError::Other(format!(
                "Original subtitle file not found: {:?}",
                original_file
            )));
        }
        
        // Create workspace directory
        let workspace_id = Uuid::new_v4();
        let base_temp = std::env::temp_dir();
        let workspace_dir = base_temp
            .join("animehub")
            .join("subtitle_workspaces")
            .join(workspace_id.to_string());
        
        fs::create_dir_all(&workspace_dir)
            .map_err(|e| AppError::Io(e))?;
        
        // Copy original file to workspace
        let filename = original_file
            .file_name()
            .ok_or_else(|| AppError::Other("Invalid filename".to_string()))?;
        let working_file = workspace_dir.join(filename);
        
        fs::copy(&original_file, &working_file)
            .map_err(|e| AppError::Io(e))?;
        
        Ok(Self {
            id: workspace_id,
            workspace_dir,
            original_file,
            working_file,
            created_at: Utc::now(),
            is_cleaned: false,
        })
    }
    
    /// Get the path to the working file
    /// 
    /// This is the file that should be modified during transformations.
    pub fn working_file_path(&self) -> &Path {
        &self.working_file
    }
    
    /// Get the path to the original file (read-only)
    pub fn original_file_path(&self) -> &Path {
        &self.original_file
    }
    
    /// Check if the workspace is still valid
    pub fn is_valid(&self) -> bool {
        !self.is_cleaned && self.workspace_dir.exists()
    }
    
    /// Clean up the workspace
    /// 
    /// This removes the temporary directory and all its contents.
    /// 
    /// CRITICAL: This NEVER touches the original file.
    /// CRITICAL: This should only be called after success or explicit cancellation.
    pub fn cleanup(&mut self) -> AppResult<()> {
        if self.is_cleaned {
            return Ok(()); // Already cleaned
        }
        
        // Verify we're only deleting inside temp directory
        let temp_base = std::env::temp_dir().join("animehub").join("subtitle_workspaces");
        if !self.workspace_dir.starts_with(&temp_base) {
            return Err(AppError::Other(
                "Workspace directory is not in expected temp location".to_string()
            ));
        }
        
        // Delete workspace directory
        if self.workspace_dir.exists() {
            fs::remove_dir_all(&self.workspace_dir)
                .map_err(|e| AppError::Io(e))?;
        }
        
        self.is_cleaned = true;
        Ok(())
    }
    
    /// Copy the working file to a destination
    /// 
    /// This is used to save the transformed subtitle to its final location.
    pub fn copy_working_file_to(&self, destination: &Path) -> AppResult<()> {
        if !self.is_valid() {
            return Err(AppError::Other("Workspace is not valid".to_string()));
        }
        
        // Create parent directory if needed
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::Io(e))?;
        }
        
        fs::copy(&self.working_file, destination)
            .map_err(|e| AppError::Io(e))?;
        
        Ok(())
    }
}

impl Drop for SubtitleWorkspace {
    fn drop(&mut self) {
        // Attempt cleanup on drop, but don't panic if it fails
        if !self.is_cleaned {
            let _ = self.cleanup();
        }
    }
}

// ============================================================================
// WORKSPACE EVENTS
// ============================================================================

/// Events specific to workspace lifecycle
use crate::events::types::DomainEvent;
use serde::{Deserialize, Serialize};

/// Emitted when a subtitle workspace is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleWorkspaceCreated {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub workspace_id: Uuid,
    pub subtitle_id: Uuid,
}

impl SubtitleWorkspaceCreated {
    pub fn new(workspace_id: Uuid, subtitle_id: Uuid) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            workspace_id,
            subtitle_id,
        }
    }
}

impl DomainEvent for SubtitleWorkspaceCreated {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "SubtitleWorkspaceCreated" }
}

/// Emitted when a subtitle workspace is cleaned up
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleWorkspaceCleaned {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub workspace_id: Uuid,
}

impl SubtitleWorkspaceCleaned {
    pub fn new(workspace_id: Uuid) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            workspace_id,
        }
    }
}

impl DomainEvent for SubtitleWorkspaceCleaned {
    fn event_id(&self) -> Uuid { self.event_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_type(&self) -> &'static str { "SubtitleWorkspaceCleaned" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    
    fn create_test_subtitle() -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!("test_subtitle_{}.srt", Uuid::new_v4()));
        
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"1\n00:00:01,000 --> 00:00:03,000\nTest subtitle\n").unwrap();
        
        test_file
    }
    
    #[test]
    fn test_workspace_creation() {
        let original = create_test_subtitle();
        let workspace = SubtitleWorkspace::new(original.clone()).unwrap();
        
        assert!(workspace.is_valid());
        assert!(workspace.workspace_dir.exists());
        assert!(workspace.working_file.exists());
        assert_eq!(workspace.original_file, original);
        
        // Cleanup
        let _ = fs::remove_file(&original);
    }
    
    #[test]
    fn test_workspace_cleanup() {
        let original = create_test_subtitle();
        let mut workspace = SubtitleWorkspace::new(original.clone()).unwrap();
        
        let workspace_dir = workspace.workspace_dir.clone();
        
        workspace.cleanup().unwrap();
        
        assert!(workspace.is_cleaned);
        assert!(!workspace_dir.exists());
        assert!(original.exists()); // Original untouched
        
        // Cleanup
        let _ = fs::remove_file(&original);
    }
    
    #[test]
    fn test_copy_working_file() {
        let original = create_test_subtitle();
        let workspace = SubtitleWorkspace::new(original.clone()).unwrap();
        
        let dest = std::env::temp_dir().join(format!("dest_{}.srt", Uuid::new_v4()));
        
        workspace.copy_working_file_to(&dest).unwrap();
        
        assert!(dest.exists());
        
        // Cleanup
        let _ = fs::remove_file(&original);
        let _ = fs::remove_file(&dest);
    }
    
    #[test]
    fn test_original_never_modified() {
        let original = create_test_subtitle();
        let original_content = fs::read_to_string(&original).unwrap();
        
        let mut workspace = SubtitleWorkspace::new(original.clone()).unwrap();
        
        // Modify working file
        fs::write(&workspace.working_file, "MODIFIED").unwrap();
        
        // Cleanup
        workspace.cleanup().unwrap();
        
        // Original should be unchanged
        let final_content = fs::read_to_string(&original).unwrap();
        assert_eq!(original_content, final_content);
        
        // Cleanup
        let _ = fs::remove_file(&original);
    }
}