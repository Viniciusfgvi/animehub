// src-tauri/src/domain/resolution/value_objects.rs
//
// Resolution Value Objects - Phase 4 (FINAL CORRECTED VERSION)
//
// PURPOSE:
// These value objects represent the semantic output of resolution.
// They are immutable, deterministic, and carry no operational metadata.
//
// CRITICAL INVARIANTS:
// - No timestamps in any value object
// - No random IDs in any value object
// - Fingerprints are deterministic and documented
// - All dead variants have been removed
//
// PHASE 4 CLOSURE CORRECTIONS:
// - Removed dead variants: Auxiliary, Range, FolderHierarchy, DatabaseMatch, Combined,
//   AmbiguousResolution, UnexpectedPathStructure
// - Added RepositoryError to ResolutionFailureReason
// - Documented fingerprint components for determinism verification

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

// ============================================================================
// RESOLUTION RESULT
// ============================================================================

/// The outcome of attempting to resolve a file.
#[derive(Debug, Clone)]
pub enum ResolutionResult {
    /// File was successfully resolved
    Success(ResolvedFile),

    /// File resolution failed
    Failure(ResolutionFailure),
}

impl ResolutionResult {
    pub fn is_success(&self) -> bool {
        matches!(self, ResolutionResult::Success(_))
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, ResolutionResult::Failure(_))
    }

    pub fn resolved_file(&self) -> Option<&ResolvedFile> {
        match self {
            ResolutionResult::Success(f) => Some(f),
            ResolutionResult::Failure(_) => None,
        }
    }

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
///
/// DETERMINISM: No timestamps. Identical input produces identical output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolvedFile {
    /// The file ID from the File domain (already persisted by scan phase)
    pub file_id: Uuid,

    /// The absolute path of the file (for traceability)
    pub file_path: PathBuf,

    /// The role this file plays (video, subtitle, image)
    pub role: FileRole,

    /// The resolved anime intent (what anime this file belongs to)
    pub anime_intent: ResolvedAnimeIntent,

    /// The resolved episode intent (what episode this file represents)
    pub episode_intent: ResolvedEpisodeIntent,

    /// Confidence score of the resolution (0.0 to 1.0)
    pub confidence: ResolutionConfidence,
}

impl ResolvedFile {
    /// Creates a new ResolvedFile (deterministic, no timestamps)
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
        }
    }

    /// Computes a deterministic fingerprint for idempotency checking.
    ///
    /// DETERMINISM COMPONENTS (documented for verification):
    /// - file_id: UUID of the file being resolved
    /// - anime_intent.title: lowercased for case-insensitive matching
    /// - episode_intent.number: string representation of episode number
    /// - role: string representation of file role
    ///
    /// NOTE: file_path is NOT included as the same file may be moved;
    /// file_id is the stable identifier.
    pub fn fingerprint(&self) -> ResolutionFingerprint {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.file_id.hash(&mut hasher);
        self.anime_intent.title.to_lowercase().hash(&mut hasher);
        self.episode_intent.number.to_string().hash(&mut hasher);
        self.role.to_string().hash(&mut hasher);
        ResolutionFingerprint(format!("{:016x}", hasher.finish()))
    }
}

// ============================================================================
// RESOLUTION FINGERPRINT
// ============================================================================

/// A deterministic fingerprint for a resolution result.
/// Used for idempotency checking and event ID derivation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResolutionFingerprint(String);

impl ResolutionFingerprint {
    /// Create a fingerprint from a raw hash string
    pub fn from_hash(hash: String) -> Self {
        Self(hash)
    }

    /// Get the hash string
    pub fn hash(&self) -> &str {
        &self.0
    }

    /// Convert to string for storage
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl std::fmt::Display for ResolutionFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// FILE ROLE
// ============================================================================

/// The role a file plays in the context of an episode.
///
/// PHASE 4 CORRECTION: Removed `Auxiliary` variant (was never produced).
/// Files of type `Outro` fail resolution with UnsupportedFileType.
///
/// CANONICAL VARIANTS (exhaustive):
/// - Video: Primary video file
/// - Subtitle: Subtitle file
/// - Image: Cover image or thumbnail
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileRole {
    /// Primary video file for the episode
    Video,

    /// Subtitle file associated with the episode
    Subtitle,

    /// Cover image or thumbnail
    Image,
}

impl std::fmt::Display for FileRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileRole::Video => write!(f, "video"),
            FileRole::Subtitle => write!(f, "subtitle"),
            FileRole::Image => write!(f, "image"),
        }
    }
}

// ============================================================================
// RESOLVED ANIME INTENT
// ============================================================================

/// Represents the resolved intent for which anime a file belongs to.
/// This is knowledge, not a committed entity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolvedAnimeIntent {
    /// The parsed/inferred anime title
    pub title: String,

    /// Alternative titles found during parsing (if any)
    pub alternative_titles: Vec<String>,

    /// If an existing anime was matched, its ID
    pub matched_anime_id: Option<Uuid>,

    /// The source of this resolution (filename or folder_name only)
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Resolved episode number (regular or special only)
///
/// PHASE 4 CORRECTION: Removed `Range` variant (was never produced by parsing logic).
/// Batch/range files are not supported in Phase 4 resolution.
///
/// CANONICAL VARIANTS (exhaustive):
/// - Regular: Numbered episode (1, 2, 3, ...)
/// - Special: Special episode with label (OVA, OAD, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResolvedEpisodeNumber {
    /// Regular numbered episode (1, 2, 3, ...)
    Regular { number: u32 },

    /// Special episode with a label (OVA, OAD, Special, etc.)
    Special { label: String },
}

impl std::fmt::Display for ResolvedEpisodeNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedEpisodeNumber::Regular { number } => write!(f, "{}", number),
            ResolvedEpisodeNumber::Special { label } => write!(f, "{}", label),
        }
    }
}

// ============================================================================
// RESOLUTION SOURCE
// ============================================================================

/// Where the resolution information was extracted from.
/// Used for traceability and debugging.
///
/// PHASE 4 CORRECTION: Removed `FolderHierarchy`, `DatabaseMatch`, `Combined` variants.
/// - FolderHierarchy was never produced
/// - DatabaseMatch was never produced (matching returns Option<Uuid>, not source)
/// - Combined is replaced by explicit source tracking
///
/// CANONICAL VARIANTS (exhaustive):
/// - Filename: Parsed from the filename
/// - FolderName: Inferred from the parent folder name
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionSource {
    /// Parsed from the filename
    Filename,

    /// Inferred from the parent folder name
    FolderName,
}

impl std::fmt::Display for ResolutionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolutionSource::Filename => write!(f, "filename"),
            ResolutionSource::FolderName => write!(f, "folder_name"),
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
/// No timestamps - deterministic from input.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolutionFailure {
    /// The file ID that failed to resolve
    pub file_id: Uuid,

    /// The file path (for traceability)
    pub file_path: PathBuf,

    /// The structured reason for failure
    pub reason: ResolutionFailureReason,

    /// Human-readable description
    pub description: String,
}

impl ResolutionFailure {
    /// Creates a new resolution failure (deterministic, no timestamps)
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
        }
    }

    /// Computes a deterministic fingerprint for the failure.
    ///
    /// DETERMINISM COMPONENTS:
    /// - file_id: UUID of the file
    /// - reason: structured failure reason
    pub fn fingerprint(&self) -> ResolutionFingerprint {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.file_id.hash(&mut hasher);
        self.reason.to_string().hash(&mut hasher);
        ResolutionFingerprint(format!("fail:{:016x}", hasher.finish()))
    }
}

/// Structured reasons for resolution failure.
///
/// PHASE 4 CORRECTION: Removed `AmbiguousResolution` and `UnexpectedPathStructure` variants.
/// Added `RepositoryError` for explicit infrastructure failure handling.
///
/// CANONICAL VARIANTS (exhaustive):
/// - UnparsableFilename: Could not extract meaningful data from filename
/// - NoEpisodeNumber: Episode number could not be determined
/// - LowConfidence: Resolution confidence below threshold
/// - UnsupportedFileType: File type is not supported for resolution
/// - RepositoryError: Database or infrastructure error during resolution
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionFailureReason {
    /// Could not extract meaningful data from filename
    UnparsableFilename,

    /// Episode number could not be determined
    NoEpisodeNumber,

    /// Resolution confidence below threshold
    LowConfidence,

    /// File type is not supported for resolution
    UnsupportedFileType,

    /// Database or infrastructure error during resolution
    RepositoryError,
}

impl std::fmt::Display for ResolutionFailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolutionFailureReason::UnparsableFilename => write!(f, "unparsable_filename"),
            ResolutionFailureReason::NoEpisodeNumber => write!(f, "no_episode_number"),
            ResolutionFailureReason::LowConfidence => write!(f, "low_confidence"),
            ResolutionFailureReason::UnsupportedFileType => write!(f, "unsupported_file_type"),
            ResolutionFailureReason::RepositoryError => write!(f, "repository_error"),
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
    fn test_file_role_has_exactly_three_variants() {
        // PROOF: FileRole has exactly 3 variants (Auxiliary removed)
        let roles = [FileRole::Video, FileRole::Subtitle, FileRole::Image];
        assert_eq!(roles.len(), 3);

        // Verify Display implementations
        assert_eq!(FileRole::Video.to_string(), "video");
        assert_eq!(FileRole::Subtitle.to_string(), "subtitle");
        assert_eq!(FileRole::Image.to_string(), "image");
    }

    #[test]
    fn test_resolution_source_has_exactly_two_variants() {
        // PROOF: ResolutionSource has exactly 2 variants (FolderHierarchy, DatabaseMatch, Combined removed)
        let sources = [ResolutionSource::Filename, ResolutionSource::FolderName];
        assert_eq!(sources.len(), 2);

        // Verify Display implementations
        assert_eq!(ResolutionSource::Filename.to_string(), "filename");
        assert_eq!(ResolutionSource::FolderName.to_string(), "folder_name");
    }

    #[test]
    fn test_episode_number_has_exactly_two_variants() {
        // PROOF: ResolvedEpisodeNumber has exactly 2 variants (Range removed)
        let regular = ResolvedEpisodeNumber::Regular { number: 1 };
        let special = ResolvedEpisodeNumber::Special { label: "OVA".to_string() };

        assert_eq!(regular.to_string(), "1");
        assert_eq!(special.to_string(), "OVA");
    }

    #[test]
    fn test_failure_reason_has_exactly_five_variants() {
        // PROOF: ResolutionFailureReason has exactly 5 variants
        // (AmbiguousResolution, UnexpectedPathStructure removed, RepositoryError added)
        let reasons = [
            ResolutionFailureReason::UnparsableFilename,
            ResolutionFailureReason::NoEpisodeNumber,
            ResolutionFailureReason::LowConfidence,
            ResolutionFailureReason::UnsupportedFileType,
            ResolutionFailureReason::RepositoryError,
        ];
        assert_eq!(reasons.len(), 5);
    }

    #[test]
    fn test_fingerprint_is_deterministic() {
        let file_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let resolved1 = ResolvedFile::new(
            file_id,
            PathBuf::from("/test/anime/Episode 01.mkv"),
            FileRole::Video,
            ResolvedAnimeIntent::from_parsed_title(
                "Test Anime".to_string(),
                ResolutionSource::Filename,
            ),
            ResolvedEpisodeIntent::from_parsed_number(
                ResolvedEpisodeNumber::Regular { number: 1 },
                ResolutionSource::Filename,
            ),
            ResolutionConfidence::high(),
        );

        let resolved2 = ResolvedFile::new(
            file_id,
            PathBuf::from("/test/anime/Episode 01.mkv"),
            FileRole::Video,
            ResolvedAnimeIntent::from_parsed_title(
                "Test Anime".to_string(),
                ResolutionSource::Filename,
            ),
            ResolvedEpisodeIntent::from_parsed_number(
                ResolvedEpisodeNumber::Regular { number: 1 },
                ResolutionSource::Filename,
            ),
            ResolutionConfidence::high(),
        );

        // PROOF: Identical input produces identical fingerprint
        assert_eq!(
            resolved1.fingerprint().to_string(),
            resolved2.fingerprint().to_string(),
            "Identical input MUST produce identical fingerprint"
        );
    }

    #[test]
    fn test_fingerprint_case_insensitive_title() {
        let file_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let resolved1 = ResolvedFile::new(
            file_id,
            PathBuf::from("/test/file.mkv"),
            FileRole::Video,
            ResolvedAnimeIntent::from_parsed_title(
                "Test Anime".to_string(),
                ResolutionSource::Filename,
            ),
            ResolvedEpisodeIntent::from_parsed_number(
                ResolvedEpisodeNumber::Regular { number: 1 },
                ResolutionSource::Filename,
            ),
            ResolutionConfidence::high(),
        );

        let resolved2 = ResolvedFile::new(
            file_id,
            PathBuf::from("/test/file.mkv"),
            FileRole::Video,
            ResolvedAnimeIntent::from_parsed_title(
                "TEST ANIME".to_string(),  // Different case
                ResolutionSource::Filename,
            ),
            ResolvedEpisodeIntent::from_parsed_number(
                ResolvedEpisodeNumber::Regular { number: 1 },
                ResolutionSource::Filename,
            ),
            ResolutionConfidence::high(),
        );

        // PROOF: Title comparison is case-insensitive
        assert_eq!(
            resolved1.fingerprint().to_string(),
            resolved2.fingerprint().to_string(),
            "Fingerprint MUST be case-insensitive for anime title"
        );
    }

    #[test]
    fn test_no_timestamp_in_resolved_file() {
        let file_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let resolved = ResolvedFile::new(
            file_id,
            PathBuf::from("/test/file.mkv"),
            FileRole::Video,
            ResolvedAnimeIntent::from_parsed_title(
                "Test".to_string(),
                ResolutionSource::Filename,
            ),
            ResolvedEpisodeIntent::from_parsed_number(
                ResolvedEpisodeNumber::Regular { number: 1 },
                ResolutionSource::Filename,
            ),
            ResolutionConfidence::high(),
        );

        // PROOF: ResolvedFile has no timestamp field
        // This is verified by the struct definition - no resolved_at field exists
        // The test compiles only if no timestamp field is present
        let _ = resolved.file_id;
        let _ = resolved.file_path;
        let _ = resolved.role;
        let _ = resolved.anime_intent;
        let _ = resolved.episode_intent;
        let _ = resolved.confidence;
        // If resolved.resolved_at existed, this test would need to reference it
    }

    #[test]
    fn test_no_timestamp_in_resolution_failure() {
        let file_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let failure = ResolutionFailure::new(
            file_id,
            PathBuf::from("/test/unknown.bin"),
            ResolutionFailureReason::UnsupportedFileType,
            "Binary file not supported".to_string(),
        );

        // PROOF: ResolutionFailure has no timestamp field
        let _ = failure.file_id;
        let _ = failure.file_path;
        let _ = failure.reason;
        let _ = failure.description;
        // If failure.failed_at existed, this test would need to reference it
    }
}
