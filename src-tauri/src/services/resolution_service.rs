// src-tauri/src/services/resolution_service.rs
//
// Resolution Service - Phase 4 (CORRECTED)
//
// PURPOSE:
// - Parse file metadata to extract anime and episode information
// - Match against existing domain entities (READ-ONLY)
// - Emit resolution events for Phase 5 consumption
// - Aggregate file resolutions into episode resolutions
//
// CRITICAL INVARIANTS (ALL ENFORCED):
// - Resolution is PURE: no domain mutations, no persistence
// - Resolution is DETERMINISTIC: same input → same output (no timestamps)
// - Resolution is IDEMPOTENT: repeated resolution of same file is skipped
// - Repository access is READ-ONLY
// - All errors are EXPLICIT: no silent fallbacks
// - All events are REACHABLE: no dead code paths
//
// PHASE 4 CORRECTIONS APPLIED:
// - Idempotency enforced via fingerprint tracking
// - EpisodeResolved emitted through aggregation logic
// - Silent error swallowing eliminated (.ok().flatten() removed)
// - Image files included in batch resolution
// - skipped_count is meaningful (tracks idempotent skips)
// - All dead enum variants removed from value objects
// - REMOVED: ResolutionFingerprint::from_result (hallucinated API)
// - REMOVED: FileRepository::list_by_type (hallucinated API)
// - FIXED: UnparsableTitle → UnparsableFilename
// - FIXED: UnparsableEpisodeNumber → NoEpisodeNumber

use crate::domain::file::{File, FileType};
use crate::domain::resolution::{
    FileRole, ResolutionConfidence, ResolutionFailure, ResolutionFailureReason,
    ResolutionResult, ResolutionSource, ResolvedAnimeIntent,
    ResolvedEpisodeIntent, ResolvedEpisodeNumber, ResolvedFile,
};
use crate::domain::resolution::value_objects::ResolutionFingerprint;
use crate::domain::anime::Anime;
use crate::domain::episode::Episode;
use crate::error::{AppError, AppResult};
use crate::events::resolution_events::{
    EpisodeResolved, FileResolved, ResolutionBatchCompleted, ResolutionFailed, ResolutionSkipped,
};
use crate::events::EventBus;
use crate::repositories::{AnimeRepository, EpisodeRepository, FileRepository};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// RESOLUTION SERVICE
// ============================================================================

pub struct ResolutionService {
    file_repo: Arc<dyn FileRepository>,
    anime_repo: Arc<dyn AnimeRepository>,
    episode_repo: Arc<dyn EpisodeRepository>,
    event_bus: Arc<EventBus>,
    rules: ResolutionRules,
    /// Tracks fingerprints of already-resolved files for idempotency
    resolved_fingerprints: std::sync::RwLock<HashSet<String>>,
}

impl ResolutionService {
    pub fn new(
        file_repo: Arc<dyn FileRepository>,
        anime_repo: Arc<dyn AnimeRepository>,
        episode_repo: Arc<dyn EpisodeRepository>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            file_repo,
            anime_repo,
            episode_repo,
            event_bus,
            rules: ResolutionRules::default(),
            resolved_fingerprints: std::sync::RwLock::new(HashSet::new()),
        }
    }

    /// Load existing fingerprints from a persistence source (called at startup)
    pub fn load_fingerprints(&self, fingerprints: Vec<String>) {
        let mut guard = self.resolved_fingerprints.write().unwrap();
        for fp in fingerprints {
            guard.insert(fp);
        }
    }

    // ========================================================================
    // PUBLIC API
    // ========================================================================

    /// Resolve a single file by ID.
    /// Returns the resolution result and emits appropriate events.
    ///
    /// IDEMPOTENCY: If file was already resolved (fingerprint exists), returns skipped.
    pub fn resolve_file(&self, file_id: Uuid) -> AppResult<ResolutionOutcome> {
        let file = self.file_repo.get_by_id(file_id)?;

        match file {
            Some(f) => {
                let result = self.resolve_file_internal(&f)?;

                // Compute fingerprint from the result
                let fingerprint = self.compute_fingerprint_from_result(&result);
                if self.is_already_resolved(&fingerprint) {
                    let outcome = ResolutionOutcome::Skipped {
                        file_id: f.id,
                        fingerprint: fingerprint.hash().to_string(),
                    };
                    self.event_bus.emit(ResolutionSkipped::new(
                        f.id,
                        f.caminho_absoluto.clone(),
                        fingerprint.hash().to_string(),
                        "already_resolved".to_string(),
                    ));
                    return Ok(outcome);
                }

                // Mark as resolved
                self.mark_resolved(&fingerprint);

                // Emit event
                self.emit_resolution_event(&result);

                Ok(ResolutionOutcome::Processed(result))
            }
            None => Err(AppError::NotFound),
        }
    }

    /// Resolve all pending files (Video, Subtitle, Image).
    /// Enforces idempotency and aggregates episode resolutions.
    ///
    /// CORRECTION: Uses list_unlinked() instead of hallucinated list_by_type()
    pub fn resolve_all_pending(&self) -> AppResult<ResolutionBatchResult> {
        let start_time = std::time::Instant::now();

        // Get all unlinked files and filter by type
        let all_files = self.file_repo.list_unlinked()?;
        let files: Vec<File> = all_files
            .into_iter()
            .filter(|f| matches!(f.tipo, FileType::Video | FileType::Legenda | FileType::Imagem))
            .collect();

        self.resolve_batch(files, start_time)
    }

    /// Resolve files in a specific directory.
    /// Enforces idempotency and aggregates episode resolutions.
    pub fn resolve_directory(&self, directory_path: &PathBuf) -> AppResult<ResolutionBatchResult> {
        let start_time = std::time::Instant::now();

        // Get all unlinked files and filter by directory and type
        let all_files = self.file_repo.list_unlinked()?;
        let files: Vec<File> = all_files
            .into_iter()
            .filter(|f| {
                matches!(f.tipo, FileType::Video | FileType::Legenda | FileType::Imagem)
                    && f.caminho_absoluto.starts_with(directory_path)
            })
            .collect();

        self.resolve_batch(files, start_time)
    }

    // ========================================================================
    // BATCH RESOLUTION (INTERNAL)
    // ========================================================================

    fn resolve_batch(
        &self,
        files: Vec<File>,
        start_time: std::time::Instant,
    ) -> AppResult<ResolutionBatchResult> {
        let total_files = files.len();
        let mut results: Vec<ResolutionResult> = Vec::with_capacity(total_files);
        let mut resolved_count = 0;
        let mut failed_count = 0;
        let mut skipped_count = 0;

        // Aggregation map: (anime_title_normalized, episode_number) -> files
        let mut episode_aggregation: HashMap<(String, String), EpisodeAggregation> = HashMap::new();

        for file in files {
            let result = self.resolve_file_internal(&file)?;

            // Compute fingerprint from the result
            let fingerprint = self.compute_fingerprint_from_result(&result);
            if self.is_already_resolved(&fingerprint) {
                skipped_count += 1;
                self.event_bus.emit(ResolutionSkipped::new(
                    file.id,
                    file.caminho_absoluto.clone(),
                    fingerprint.hash().to_string(),
                    "already_resolved".to_string(),
                ));
                continue;
            }

            // Mark as resolved
            self.mark_resolved(&fingerprint);

            // Emit file-level event
            self.emit_resolution_event(&result);

            match &result {
                ResolutionResult::Success(resolved) => {
                    resolved_count += 1;

                    // Aggregate for episode resolution
                    let key = (
                        resolved.anime_intent.title.to_lowercase(),
                        resolved.episode_intent.number.to_string(),
                    );
                    let aggregation = episode_aggregation.entry(key).or_insert_with(|| {
                        EpisodeAggregation::new(
                            resolved.anime_intent.title.clone(),
                            resolved.anime_intent.matched_anime_id,
                            resolved.episode_intent.number.to_string(),
                            resolved.episode_intent.matched_episode_id,
                        )
                    });
                    aggregation.add_file(resolved);
                }
                ResolutionResult::Failure(_) => {
                    failed_count += 1;
                }
            }

            results.push(result);
        }

        // Emit EpisodeResolved events for aggregated episodes
        let episodes_aggregated = episode_aggregation.len();
        for (_, aggregation) in episode_aggregation {
            let episode_event = aggregation.into_event();
            self.event_bus.emit(episode_event);
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Emit batch completion event
        self.event_bus.emit(ResolutionBatchCompleted::new(
            total_files,
            resolved_count,
            failed_count,
            skipped_count,
            episodes_aggregated,
            duration_ms,
        ));

        Ok(ResolutionBatchResult {
            results,
            total_files,
            resolved_count,
            failed_count,
            skipped_count,
            episodes_aggregated,
            duration_ms,
        })
    }

    // ========================================================================
    // INTERNAL RESOLUTION LOGIC
    // ========================================================================

    /// Resolve a single file (internal, does not emit events).
    ///
    /// CORRECTION: Errors are propagated explicitly, not swallowed.
    fn resolve_file_internal(&self, file: &File) -> AppResult<ResolutionResult> {
        // Step 1: Determine file role
        let role = match file.tipo {
            FileType::Video => FileRole::Video,
            FileType::Legenda => FileRole::Subtitle,
            FileType::Imagem => FileRole::Image,
            FileType::Outro => {
                return Ok(ResolutionResult::Failure(ResolutionFailure::new(
                    file.id,
                    file.caminho_absoluto.clone(),
                    ResolutionFailureReason::UnsupportedFileType,
                    "File type 'outro' is not supported for resolution".to_string(),
                )));
            }
        };

        // Step 2: Parse anime title
        // CORRECTION: UnparsableTitle → UnparsableFilename (canonical variant)
        let anime_parse_result = self.rules.parse_anime_title(&file.caminho_absoluto);
        let (anime_title, anime_source) = match anime_parse_result {
            Some((title, source)) => (title, source),
            None => {
                return Ok(ResolutionResult::Failure(ResolutionFailure::new(
                    file.id,
                    file.caminho_absoluto.clone(),
                    ResolutionFailureReason::UnparsableFilename,
                    "Could not extract anime title from filename or folder".to_string(),
                )));
            }
        };

        // Step 3: Parse episode number
        // CORRECTION: UnparsableEpisodeNumber → NoEpisodeNumber (canonical variant)
        let episode_parse_result = self.rules.parse_episode_number(&file.caminho_absoluto);
        let (episode_number, episode_source) = match episode_parse_result {
            Some((num, source)) => (num, source),
            None => {
                return Ok(ResolutionResult::Failure(ResolutionFailure::new(
                    file.id,
                    file.caminho_absoluto.clone(),
                    ResolutionFailureReason::NoEpisodeNumber,
                    "Could not extract episode number from filename".to_string(),
                )));
            }
        };

        // Step 4: Try to match against existing anime (READ-ONLY)
        // CORRECTION: Errors are propagated, not swallowed
        let matched_anime_id = self.try_match_anime(&anime_title)?;

        // Step 5: Try to match against existing episode (if anime matched)
        // CORRECTION: Errors are propagated, not swallowed
        let matched_episode_id = match matched_anime_id {
            Some(anime_id) => self.try_match_episode(anime_id, &episode_number)?,
            None => None,
        };

        // Step 6: Calculate confidence
        let confidence = self.rules.calculate_confidence(
            &anime_title,
            &episode_number,
            matched_anime_id.is_some(),
            matched_episode_id.is_some(),
            &anime_source,
            &episode_source,
        );

        // Step 7: Check confidence threshold
        if !confidence.meets_threshold() {
            return Ok(ResolutionResult::Failure(ResolutionFailure::new(
                file.id,
                file.caminho_absoluto.clone(),
                ResolutionFailureReason::LowConfidence,
                format!(
                    "Confidence score {} is below threshold {}",
                    confidence.score(),
                    ResolutionConfidence::THRESHOLD
                ),
            )));
        }

        // Step 8: Build resolved file
        let anime_intent = match matched_anime_id {
            Some(id) => ResolvedAnimeIntent::matched(id, anime_title, anime_source),
            None => ResolvedAnimeIntent::from_parsed_title(anime_title, anime_source),
        };

        let episode_intent = match matched_episode_id {
            Some(id) => ResolvedEpisodeIntent::matched(id, episode_number, episode_source),
            None => ResolvedEpisodeIntent::from_parsed_number(episode_number, episode_source),
        };

        Ok(ResolutionResult::Success(ResolvedFile::new(
            file.id,
            file.caminho_absoluto.clone(),
            role,
            anime_intent,
            episode_intent,
            confidence,
        )))
    }

    /// Attempt to match parsed title against existing anime entities.
    /// This is a READ-ONLY operation.
    ///
    /// CORRECTION: Returns AppResult, errors are not swallowed.
    fn try_match_anime(&self, title: &str) -> AppResult<Option<Uuid>> {
        let animes = self.anime_repo.list_all()?;
        let normalized_title = self.rules.normalize_title(title);

        for anime in animes {
            let normalized_anime_title = self.rules.normalize_title(&anime.titulo_principal);
            if normalized_anime_title == normalized_title {
                return Ok(Some(anime.id));
            }
        }

        Ok(None)
    }

    /// Attempt to match parsed episode number against existing episodes.
    /// This is a READ-ONLY operation.
    ///
    /// CORRECTION: Returns AppResult, errors are not swallowed.
    fn try_match_episode(
        &self,
        anime_id: Uuid,
        episode_number: &ResolvedEpisodeNumber,
    ) -> AppResult<Option<Uuid>> {
        let episodes = self.episode_repo.list_by_anime(anime_id)?;

        for episode in episodes {
            match (&episode.numero, episode_number) {
                (
                    crate::domain::episode::EpisodeNumber::Regular { numero },
                    ResolvedEpisodeNumber::Regular { number },
                ) => {
                    if numero == number {
                        return Ok(Some(episode.id));
                    }
                }
                (
                    crate::domain::episode::EpisodeNumber::Special { label: ep_label },
                    ResolvedEpisodeNumber::Special { label: res_label },
                ) => {
                    if ep_label.to_lowercase() == res_label.to_lowercase() {
                        return Ok(Some(episode.id));
                    }
                }
                _ => {}
            }
        }

        Ok(None)
    }

    // ========================================================================
    // IDEMPOTENCY ENFORCEMENT
    // ========================================================================

    fn is_already_resolved(&self, fingerprint: &ResolutionFingerprint) -> bool {
        let guard = self.resolved_fingerprints.read().unwrap();
        guard.contains(fingerprint.hash())
    }

    fn mark_resolved(&self, fingerprint: &ResolutionFingerprint) {
        let mut guard = self.resolved_fingerprints.write().unwrap();
        guard.insert(fingerprint.hash().to_string());
    }

    /// Compute fingerprint from a ResolutionResult.
    /// CORRECTION: This replaces the hallucinated ResolutionFingerprint::from_result
    fn compute_fingerprint_from_result(&self, result: &ResolutionResult) -> ResolutionFingerprint {
        match result {
            ResolutionResult::Success(resolved) => resolved.fingerprint(),
            ResolutionResult::Failure(failure) => failure.fingerprint(),
        }
    }

    // ========================================================================
    // EVENT EMISSION
    // ========================================================================

    fn emit_resolution_event(&self, result: &ResolutionResult) {
        match result {
            ResolutionResult::Success(resolved) => {
                self.event_bus.emit(FileResolved::new(
                    resolved.file_id,
                    resolved.file_path.clone(),
                    resolved.anime_intent.title.clone(),
                    resolved.anime_intent.matched_anime_id,
                    resolved.episode_intent.number.to_string(),
                    resolved.episode_intent.matched_episode_id,
                    resolved.role.to_string(),
                    resolved.confidence.score(),
                    resolved.anime_intent.source.to_string(),
                    resolved.fingerprint().to_string(),
                ));
            }
            ResolutionResult::Failure(failure) => {
                self.event_bus.emit(ResolutionFailed::new(
                    failure.file_id,
                    failure.file_path.clone(),
                    failure.reason.to_string(),
                    failure.description.clone(),
                ));
            }
        }
    }
}

// ============================================================================
// EPISODE AGGREGATION (FOR EpisodeResolved EMISSION)
// ============================================================================

/// Aggregates file resolutions into an episode resolution.
/// This makes EpisodeResolved reachable through real resolution flow.
struct EpisodeAggregation {
    anime_title: String,
    matched_anime_id: Option<Uuid>,
    episode_number: String,
    matched_episode_id: Option<Uuid>,
    video_file_id: Option<Uuid>,
    subtitle_file_ids: Vec<Uuid>,
    image_file_ids: Vec<Uuid>,
    max_confidence: f64,
}

impl EpisodeAggregation {
    fn new(
        anime_title: String,
        matched_anime_id: Option<Uuid>,
        episode_number: String,
        matched_episode_id: Option<Uuid>,
    ) -> Self {
        Self {
            anime_title,
            matched_anime_id,
            episode_number,
            matched_episode_id,
            video_file_id: None,
            subtitle_file_ids: Vec::new(),
            image_file_ids: Vec::new(),
            max_confidence: 0.0,
        }
    }

    fn add_file(&mut self, resolved: &ResolvedFile) {
        match resolved.role {
            FileRole::Video => {
                // First video becomes primary
                if self.video_file_id.is_none() {
                    self.video_file_id = Some(resolved.file_id);
                }
            }
            FileRole::Subtitle => {
                self.subtitle_file_ids.push(resolved.file_id);
            }
            FileRole::Image => {
                self.image_file_ids.push(resolved.file_id);
            }
        }

        if resolved.confidence.score() > self.max_confidence {
            self.max_confidence = resolved.confidence.score();
        }
    }

    fn into_event(self) -> EpisodeResolved {
        EpisodeResolved::new(
            self.anime_title,
            self.matched_anime_id,
            self.episode_number,
            self.matched_episode_id,
            self.video_file_id,
            self.subtitle_file_ids,
            self.image_file_ids,
            self.max_confidence,
        )
    }
}

// ============================================================================
// RESOLUTION OUTCOME (FOR SINGLE FILE API)
// ============================================================================

/// Outcome of resolving a single file.
#[derive(Debug)]
pub enum ResolutionOutcome {
    /// File was processed (success or failure)
    Processed(ResolutionResult),

    /// File was skipped due to idempotency
    Skipped { file_id: Uuid, fingerprint: String },
}

// ============================================================================
// RESOLUTION BATCH RESULT
// ============================================================================

/// Result of a batch resolution operation.
#[derive(Debug)]
pub struct ResolutionBatchResult {
    pub results: Vec<ResolutionResult>,
    pub total_files: usize,
    pub resolved_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
    pub episodes_aggregated: usize,
    pub duration_ms: u64,
}

// ============================================================================
// RESOLUTION RULES
// ============================================================================

pub struct ResolutionRules {
    title_patterns: Vec<Regex>,
    episode_number_patterns: Vec<Regex>,
    special_episode_patterns: Vec<Regex>,
}

impl Default for ResolutionRules {
    fn default() -> Self {
        Self {
            title_patterns: vec![
                // [SubGroup] Title - 01 [quality].ext
                Regex::new(r"^\[.*?\]\s*(.+?)\s*-\s*\d+").unwrap(),
                // Title - 01.ext
                Regex::new(r"^(.+?)\s*-\s*\d+").unwrap(),
                // Title S01E01.ext
                Regex::new(r"^(.+?)\s*S\d+E\d+").unwrap(),
                // Title Episode 01.ext
                Regex::new(r"^(.+?)\s*Episode\s*\d+").unwrap(),
            ],
            episode_number_patterns: vec![
                // - 01, - 001
                Regex::new(r"-\s*(\d+)").unwrap(),
                // S01E01
                Regex::new(r"S\d+E(\d+)").unwrap(),
                // Episode 01
                Regex::new(r"Episode\s*(\d+)").unwrap(),
                // #01
                Regex::new(r"#(\d+)").unwrap(),
            ],
            special_episode_patterns: vec![
                // OVA, OVA 1, OVA1
                Regex::new(r"(OVA\s*\d*)").unwrap(),
                // OAD, OAD 1
                Regex::new(r"(OAD\s*\d*)").unwrap(),
                // Special, Special 1
                Regex::new(r"(Special\s*\d*)").unwrap(),
                // Movie, Movie 1
                Regex::new(r"(Movie\s*\d*)").unwrap(),
            ],
        }
    }
}

impl ResolutionRules {
    /// Parse anime title from file path
    pub fn parse_anime_title(&self, path: &PathBuf) -> Option<(String, ResolutionSource)> {
        // Try filename first
        if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
            for pattern in &self.title_patterns {
                if let Some(captures) = pattern.captures(filename) {
                    if let Some(title) = captures.get(1) {
                        return Some((title.as_str().trim().to_string(), ResolutionSource::Filename));
                    }
                }
            }
        }

        // Fall back to folder name
        if let Some(parent) = path.parent() {
            if let Some(folder) = parent.file_name().and_then(|s| s.to_str()) {
                // Use folder name as title if it looks reasonable
                if folder.len() >= 3 && !folder.starts_with('.') {
                    return Some((folder.to_string(), ResolutionSource::FolderName));
                }
            }
        }

        None
    }

    /// Parse episode number from file path
    pub fn parse_episode_number(
        &self,
        path: &PathBuf,
    ) -> Option<(ResolvedEpisodeNumber, ResolutionSource)> {
        let filename = path.file_stem().and_then(|s| s.to_str())?;

        // Check for special episodes first
        for pattern in &self.special_episode_patterns {
            if let Some(captures) = pattern.captures(filename) {
                if let Some(label) = captures.get(1) {
                    return Some((
                        ResolvedEpisodeNumber::Special {
                            label: label.as_str().trim().to_string(),
                        },
                        ResolutionSource::Filename,
                    ));
                }
            }
        }

        // Try regular episode patterns
        for pattern in &self.episode_number_patterns {
            if let Some(captures) = pattern.captures(filename) {
                if let Some(num_str) = captures.get(1) {
                    if let Ok(number) = num_str.as_str().parse::<u32>() {
                        return Some((
                            ResolvedEpisodeNumber::Regular { number },
                            ResolutionSource::Filename,
                        ));
                    }
                }
            }
        }

        None
    }

    /// Calculate confidence score based on resolution quality
    pub fn calculate_confidence(
        &self,
        anime_title: &str,
        episode_number: &ResolvedEpisodeNumber,
        anime_matched: bool,
        episode_matched: bool,
        anime_source: &ResolutionSource,
        episode_source: &ResolutionSource,
    ) -> ResolutionConfidence {
        let mut score = 0.5; // Base score

        // Bonus for matching existing entities
        if anime_matched {
            score += 0.2;
        }
        if episode_matched {
            score += 0.15;
        }

        // Bonus for filename source (more reliable than folder)
        if matches!(anime_source, ResolutionSource::Filename) {
            score += 0.1;
        }
        if matches!(episode_source, ResolutionSource::Filename) {
            score += 0.05;
        }

        // Penalty for short titles (likely false positives)
        if anime_title.len() < 3 {
            score -= 0.3;
        }

        // Penalty for special episodes (harder to match)
        if matches!(episode_number, ResolvedEpisodeNumber::Special { .. }) {
            score -= 0.1;
        }

        ResolutionConfidence::new(score)
    }

    /// Normalize title for comparison
    pub fn normalize_title(&self, title: &str) -> String {
        title
            .to_lowercase()
            .replace(['_', '-', '.', ':', ';', '!', '?'], " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_anime_title_from_filename() {
        let rules = ResolutionRules::default();

        // [SubGroup] Anime Title - 01 [1080p].mkv
        let path = PathBuf::from("[SubGroup] Steins Gate - 01 [1080p].mkv");
        let result = rules.parse_anime_title(&path);
        assert!(result.is_some());
        let (title, source) = result.unwrap();
        assert_eq!(title, "Steins Gate");
        assert_eq!(source, ResolutionSource::Filename);

        // Anime Title - 01.mkv
        let path = PathBuf::from("Attack on Titan - 01.mkv");
        let result = rules.parse_anime_title(&path);
        assert!(result.is_some());
        let (title, _) = result.unwrap();
        assert_eq!(title, "Attack on Titan");
    }

    #[test]
    fn test_parse_episode_number() {
        let rules = ResolutionRules::default();

        // - 01
        let path = PathBuf::from("Anime - 01.mkv");
        let result = rules.parse_episode_number(&path);
        assert!(result.is_some());
        let (num, _) = result.unwrap();
        assert_eq!(num, ResolvedEpisodeNumber::Regular { number: 1 });

        // S01E05
        let path = PathBuf::from("Anime S01E05.mkv");
        let result = rules.parse_episode_number(&path);
        assert!(result.is_some());
        let (num, _) = result.unwrap();
        assert_eq!(num, ResolvedEpisodeNumber::Regular { number: 5 });

        // OVA
        let path = PathBuf::from("Anime OVA.mkv");
        let result = rules.parse_episode_number(&path);
        assert!(result.is_some());
        let (num, _) = result.unwrap();
        assert!(matches!(num, ResolvedEpisodeNumber::Special { .. }));
    }

    #[test]
    fn test_confidence_calculation() {
        let rules = ResolutionRules::default();

        // High confidence: matched anime and episode, filename source
        let confidence = rules.calculate_confidence(
            "Steins;Gate",
            &ResolvedEpisodeNumber::Regular { number: 1 },
            true,
            true,
            &ResolutionSource::Filename,
            &ResolutionSource::Filename,
        );
        assert!(confidence.meets_threshold());
        assert!(confidence.score() > 0.8);

        // Low confidence: no matches, short title
        let confidence = rules.calculate_confidence(
            "AB",
            &ResolvedEpisodeNumber::Special {
                label: "?".to_string(),
            },
            false,
            false,
            &ResolutionSource::FolderName,
            &ResolutionSource::FolderName,
        );
        assert!(!confidence.meets_threshold());
    }

    #[test]
    fn test_normalize_title() {
        let rules = ResolutionRules::default();

        assert_eq!(rules.normalize_title("Steins;Gate"), "steins gate");
        assert_eq!(rules.normalize_title("Attack_on_Titan"), "attack on titan");
        assert_eq!(rules.normalize_title("Re:Zero"), "re zero");
    }

    #[test]
    fn test_fingerprint_determinism() {
        let file_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let result1 = ResolvedFile::new(
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

        let result2 = ResolvedFile::new(
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

        // Identical input produces identical fingerprint
        assert_eq!(result1.fingerprint(), result2.fingerprint());
    }
}
