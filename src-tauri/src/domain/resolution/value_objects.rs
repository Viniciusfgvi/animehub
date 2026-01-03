// src-tauri/src/domain/resolution/value_objects.rs
//
// Resolution Value Objects - Phase 4
//
// Pure, immutable data structures representing resolution outcomes.
// These are the bridge between raw scan data and domain mutation.
//
// CRITICAL INVARIANTS:
// - All fields are immutable (no &mut self methods)
// - No side effects
// - No I/O operations
// - Deterministic construction
// - Clone + Debug + Serialize for traceability

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

// ============================================================================
// RESOLUTION RESULT (TOP-LEVEL OUTCOME)
// ============================================================================

/// The outcome of attempting to resolve a file.
/// Either a successful resolution or an explicit failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionResult {
    /// File was successfully resolved to domain intent
    Success(ResolvedFile),
    
    /// Resolution failed with an explicit, structured reason
    Failure(ResolutionFailure),
}

impl ResolutionResult {
    /// Returns true if resolution succeeded
    pub fn is_success(&self) -> bool {
        matches!(self, ResolutionResult::Success(_))
    }
    
    /// Returns true if resolution failed
    pub fn is_failure(&self) -> bool {
        matches!(self, ResolutionResult::Failure(_))
    }
    
    /// Extracts the resolved file if successful
    pub fn resolved_file(&self) -> Option<&ResolvedFile> {
        match self {
            ResolutionResult::Success(rf) => Some(rf),
            ResolutionResult::Failure(_) => None,
        }
    }
    
    /// Extracts the failure if unsuccessful
    pub fn failure(&self) -> Option<&ResolutionFailure> {
        match self {
            ResolutionResult::Success(_) => None,
            ResolutionResult::Failure(f) => Some(f),
        }
    }
}

// ============================================================================
// RESOLVED FILE (SUCCESSFUL RESOLUTION)
// ============================================================================

/// A file that has been successfully resolved to domain intent.
/// This represents knowledge about what the file is, not a mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedFile {
    /// The file ID from the File domain (already persisted by scan phase)
    pub file_id: Uuid,
    
    /// The absolute path of the file (for traceability)
    pub file_path: PathBuf,
    
    /// The role this file plays (video, subtitle, etc.)
    pub role: FileRole,
    
    /// The resolved anime intent (what anime this file belongs to)
    pub anime_intent: ResolvedAnimeIntent,
    
    /// The resolved episode intent (what episode this file represents)
    pub episode_intent: ResolvedEpisodeIntent,
    
    /// Confidence score of the resolution (0.0 to 1.0)
    pub confidence: ResolutionConfidence,
    
    /// When this resolution was computed
    pub resolved_at: DateTime<Utc>,
}

impl ResolvedFile {
    /// Creates a new ResolvedFile
    pub fn new(
        file_id: Uuid,
        file_path: PathBuf,
        role: FileRole,
        anime_intent: ResolvedAnimeIntent,
        episode_intent: ResolvedEpisodeIntent,
        confidence: ResolutionConfidence,
    ) -> Self {
        Self {
            file_id,
            file_path,
            role,
            anime_intent,
            episode_intent,
            confidence,
            resolved_at: Utc::now(),
        }
    }
}

// ============================================================================
// FILE ROLE
// ============================================================================

/// The role a file plays in the context of an episode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileRole {
    /// Primary video file for the episode
    Video,
    
    /// Subtitle file associated with the episode
    Subtitle,
    
    /// Cover image or thumbnail
    Image,
    
    /// Other auxiliary file (e.g., NFO, sample)
    Auxiliary,
}

impl std::fmt::Display for FileRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileRole::Video => write!(f, "video"),
            FileRole::Subtitle => write!(f, "subtitle"),
            FileRole::Image => write!(f, "image"),
            FileRole::Auxiliary => write!(f, "auxiliary"),
        }
    }
}

// ============================================================================
// RESOLVED ANIME INTENT
// ============================================================================

/// Represents the resolved intent for which anime a file belongs to.
/// This is knowledge, not a committed entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedAnimeIntent {
    /// The parsed/inferred anime title
    pub title: String,
    
    /// Alternative titles found during parsing (if any)
    pub alternative_titles: Vec<String>,
    
    /// If an existing anime was matched, its ID
    pub matched_anime_id: Option<Uuid>,
    
    /// The source of this resolution (filename, folder, etc.)
    pub source: ResolutionSource,
}

impl ResolvedAnimeIntent {
    /// Creates a new anime intent from a parsed title
    pub fn from_parsed_title(title: String, source: ResolutionSource) -> Self {
        Self {
            title,
            alternative_titles: Vec::new(),
            matched_anime_id: None,
            source,
        }
    }
    
    /// Creates a new anime intent matched to an existing anime
    pub fn matched(anime_id: Uuid, title: String, source: ResolutionSource) -> Self {
        Self {
            title,
            alternative_titles: Vec::new(),
            matched_anime_id: Some(anime_id),
            source,
        }
    }
}

// ============================================================================
// RESOLVED EPISODE INTENT
// ============================================================================

/// Represents the resolved intent for which episode a file represents.
/// This is knowledge, not a committed entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedEpisodeIntent {
    /// The parsed episode number
    pub number: ResolvedEpisodeNumber,
    
    /// The parsed episode title (if found)
    pub title: Option<String>,
    
    /// If an existing episode was matched, its ID
    pub matched_episode_id: Option<Uuid>,
    
    /// The source of this resolution
    pub source: ResolutionSource,
}

impl ResolvedEpisodeIntent {
    /// Creates a new episode intent from a parsed number
    pub fn from_parsed_number(number: ResolvedEpisodeNumber, source: ResolutionSource) -> Self {
        Self {
            number,
            title: None,
            matched_episode_id: None,
            source,
        }
    }
    
    /// Creates a new episode intent matched to an existing episode
    pub fn matched(
        episode_id: Uuid,
        number: ResolvedEpisodeNumber,
        source: ResolutionSource,
    ) -> Self {
        Self {
            number,
            title: None,
            matched_episode_id: Some(episode_id),
            source,
        }
    }
}

/// Resolved episode number (regular or special)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResolvedEpisodeNumber {
    /// Regular numbered episode (1, 2, 3, ...)
    Regular { number: u32 },
    
    /// Special episode with a label (OVA, OAD, Special, etc.)
    Special { label: String },
    
    /// Range of episodes (for batch files, rare)
    Range { start: u32, end: u32 },
}

impl std::fmt::Display for ResolvedEpisodeNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedEpisodeNumber::Regular { number } => write!(f, "{}", number),
            ResolvedEpisodeNumber::Special { label } => write!(f, "{}", label),
            ResolvedEpisodeNumber::Range { start, end } => write!(f, "{}-{}", start, end),
        }
    }
}

// ============================================================================
// RESOLUTION SOURCE
// ============================================================================

/// Where the resolution information was extracted from.
/// Used for traceability and debugging.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionSource {
    /// Parsed from the filename
    Filename,
    
    /// Inferred from the parent folder name
    FolderName,
    
    /// Inferred from the folder hierarchy
    FolderHierarchy,
    
    /// Matched against existing database records
    DatabaseMatch,
    
    /// Multiple sources combined
    Combined,
}

impl std::fmt::Display for ResolutionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolutionSource::Filename => write!(f, "filename"),
            ResolutionSource::FolderName => write!(f, "folder_name"),
            ResolutionSource::FolderHierarchy => write!(f, "folder_hierarchy"),
            ResolutionSource::DatabaseMatch => write!(f, "database_match"),
            ResolutionSource::Combined => write!(f, "combined"),
        }
    }
}

// ============================================================================
// RESOLUTION CONFIDENCE
// ============================================================================

/// Confidence score for a resolution.
/// Used to determine if resolution should proceed or fail.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResolutionConfidence {
    /// Score from 0.0 (no confidence) to 1.0 (absolute certainty)
    score: f64,
}

impl ResolutionConfidence {
    /// Minimum confidence threshold for a resolution to be considered valid
    pub const THRESHOLD: f64 = 0.6;
    
    /// Creates a new confidence score, clamped to [0.0, 1.0]
    pub fn new(score: f64) -> Self {
        Self {
            score: score.clamp(0.0, 1.0),
        }
    }
    
    /// Returns the raw score
    pub fn score(&self) -> f64 {
        self.score
    }
    
    /// Returns true if confidence meets the threshold
    pub fn meets_threshold(&self) -> bool {
        self.score >= Self::THRESHOLD
    }
    
    /// High confidence (>= 0.9)
    pub fn high() -> Self {
        Self::new(0.95)
    }
    
    /// Medium confidence (>= 0.7)
    pub fn medium() -> Self {
        Self::new(0.75)
    }
    
    /// Low confidence (>= 0.5)
    pub fn low() -> Self {
        Self::new(0.55)
    }
    
    /// No confidence
    pub fn none() -> Self {
        Self::new(0.0)
    }
}

impl PartialEq for ResolutionConfidence {
    fn eq(&self, other: &Self) -> bool {
        (self.score - other.score).abs() < f64::EPSILON
    }
}

impl std::fmt::Display for ResolutionConfidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}%", self.score * 100.0)
    }
}

// ============================================================================
// RESOLUTION FAILURE
// ============================================================================

/// Represents an explicit, structured resolution failure.
/// Failures are non-fatal and traceable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionFailure {
    /// The file ID that failed to resolve
    pub file_id: Uuid,
    
    /// The file path (for traceability)
    pub file_path: PathBuf,
    
    /// The reason for failure
    pub reason: ResolutionFailureReason,
    
    /// Human-readable description
    pub description: String,
    
    /// When this failure occurred
    pub failed_at: DateTime<Utc>,
}

impl ResolutionFailure {
    /// Creates a new resolution failure
    pub fn new(
        file_id: Uuid,
        file_path: PathBuf,
        reason: ResolutionFailureReason,
        description: String,
    ) -> Self {
        Self {
            file_id,
            file_path,
            reason,
            description,
            failed_at: Utc::now(),
        }
    }
}

/// Reasons why resolution can fail
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionFailureReason {
    /// Could not parse anime title from filename or folder
    UnparsableTitle,
    
    /// Could not parse episode number from filename
    UnparsableEpisodeNumber,
    
    /// Confidence score below threshold
    LowConfidence,
    
    /// File type not supported for resolution
    UnsupportedFileType,
    
    /// Ambiguous resolution (multiple equally valid interpretations)
    AmbiguousResolution,
    
    /// File path structure does not match expected patterns
    UnexpectedPathStructure,
}

impl std::fmt::Display for ResolutionFailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolutionFailureReason::UnparsableTitle => write!(f, "unparsable_title"),
            ResolutionFailureReason::UnparsableEpisodeNumber => write!(f, "unparsable_episode_number"),
            ResolutionFailureReason::LowConfidence => write!(f, "low_confidence"),
            ResolutionFailureReason::UnsupportedFileType => write!(f, "unsupported_file_type"),
            ResolutionFailureReason::AmbiguousResolution => write!(f, "ambiguous_resolution"),
            ResolutionFailureReason::UnexpectedPathStructure => write!(f, "unexpected_path_structure"),
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_confidence_clamping() {
        let high = ResolutionConfidence::new(1.5);
        assert_eq!(high.score(), 1.0);
        
        let low = ResolutionConfidence::new(-0.5);
        assert_eq!(low.score(), 0.0);
        
        let normal = ResolutionConfidence::new(0.75);
        assert_eq!(normal.score(), 0.75);
    }

    #[test]
    fn test_resolution_confidence_threshold() {
        let above = ResolutionConfidence::new(0.7);
        assert!(above.meets_threshold());
        
        let below = ResolutionConfidence::new(0.5);
        assert!(!below.meets_threshold());
        
        let exact = ResolutionConfidence::new(0.6);
        assert!(exact.meets_threshold());
    }

    #[test]
    fn test_resolution_result_accessors() {
        let success = ResolutionResult::Success(ResolvedFile::new(
            Uuid::new_v4(),
            PathBuf::from("/test/file.mkv"),
            FileRole::Video,
            ResolvedAnimeIntent::from_parsed_title("Test Anime".to_string(), ResolutionSource::Filename),
            ResolvedEpisodeIntent::from_parsed_number(
                ResolvedEpisodeNumber::Regular { number: 1 },
                ResolutionSource::Filename,
            ),
            ResolutionConfidence::high(),
        ));
        
        assert!(success.is_success());
        assert!(!success.is_failure());
        assert!(success.resolved_file().is_some());
        assert!(success.failure().is_none());
        
        let failure = ResolutionResult::Failure(ResolutionFailure::new(
            Uuid::new_v4(),
            PathBuf::from("/test/unknown.bin"),
            ResolutionFailureReason::UnsupportedFileType,
            "Binary file not supported".to_string(),
        ));
        
        assert!(!failure.is_success());
        assert!(failure.is_failure());
        assert!(failure.resolved_file().is_none());
        assert!(failure.failure().is_some());
    }

    #[test]
    fn test_episode_number_display() {
        let regular = ResolvedEpisodeNumber::Regular { number: 5 };
        assert_eq!(regular.to_string(), "5");
        
        let special = ResolvedEpisodeNumber::Special { label: "OVA 1".to_string() };
        assert_eq!(special.to_string(), "OVA 1");
        
        let range = ResolvedEpisodeNumber::Range { start: 1, end: 3 };
        assert_eq!(range.to_string(), "1-3");
    }

    #[test]
    fn test_file_role_display() {
        assert_eq!(FileRole::Video.to_string(), "video");
        assert_eq!(FileRole::Subtitle.to_string(), "subtitle");
        assert_eq!(FileRole::Image.to_string(), "image");
        assert_eq!(FileRole::Auxiliary.to_string(), "auxiliary");
    }
}
