// src-tauri/src/services/resolution_service.rs
//
// Resolution Service - Phase 4
//
// Transforms detected files into resolved domain intent without committing state.
//
// CRITICAL RULES:
// - Consumes persisted File records (from scan phase)
// - Produces resolution results as value objects
// - Emits resolution events (FileResolved, ResolutionFailed, etc.)
// - Does NOT auto-persist Anime/Episode
// - Operates on facts already scanned, not the filesystem
// - Deterministic: same input → same output
// - Idempotent: running twice produces identical results
//
// PHASE 4 CONSTRAINTS:
// - MAY inspect repositories (read-only)
// - MAY attempt matches against existing entities
// - MAY emit confidence-bearing intent
// - MUST NOT create entities
// - MUST NOT mutate Anime/Episode state
// - MUST NOT persist links

use std::sync::Arc;
use std::path::PathBuf;
use uuid::Uuid;
use regex::Regex;

use crate::domain::file::{File, FileType};
use crate::domain::anime::Anime;
use crate::domain::resolution::{
    ResolutionResult,
    ResolvedFile,
    ResolvedAnimeIntent,
    ResolvedEpisodeIntent,
    ResolvedEpisodeNumber,
    FileRole,
    ResolutionConfidence,
    ResolutionFailure,
    ResolutionFailureReason,
    ResolutionSource,
};
use crate::repositories::{FileRepository, AnimeRepository, EpisodeRepository};
use crate::events::{
    EventBus,
    FileResolved,
    ResolutionFailed,
    ResolutionBatchCompleted,
};
use crate::error::{AppError, AppResult};

// ============================================================================
// RESOLUTION SERVICE
// ============================================================================

pub struct ResolutionService {
    file_repo: Arc<dyn FileRepository>,
    anime_repo: Arc<dyn AnimeRepository>,
    episode_repo: Arc<dyn EpisodeRepository>,
    event_bus: Arc<EventBus>,
    rules: ResolutionRules,
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
        }
    }

    /// Resolve a single file by ID
    pub fn resolve_file(&self, file_id: Uuid) -> AppResult<ResolutionResult> {
        let file = self.file_repo
            .get_by_id(file_id)?
            .ok_or(AppError::NotFound)?;

        let result = self.resolve_file_internal(&file);
        self.emit_resolution_event(&result);
        Ok(result)
    }

    /// Resolve all unresolved video and subtitle files
    pub fn resolve_all_pending(&self) -> AppResult<Vec<ResolutionResult>> {
        let start_time = std::time::Instant::now();
        
        let videos = self.file_repo.list_by_type(FileType::Video)?;
        let legenda_files = self.file_repo.list_by_type(FileType::Legenda)?;
        let mut files = videos;
        files.extend(legenda_files);
        
        let total_files = files.len();
        
        let mut results = Vec::with_capacity(total_files);
        let mut resolved_count = 0;
        let mut failed_count = 0;
        let skipped_count = 0;

        for file in files {
            let result = self.resolve_file_internal(&file);
            
            match &result {
                ResolutionResult::Success(_) => resolved_count += 1,
                ResolutionResult::Failure(_) => failed_count += 1,
            }
            
            self.emit_resolution_event(&result);
            results.push(result);
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        self.event_bus.emit(ResolutionBatchCompleted::new(
            total_files,
            resolved_count,
            failed_count,
            skipped_count,
            duration_ms,
        ));

        Ok(results)
    }

    /// Resolve files in a specific directory (by path prefix)
    pub fn resolve_directory(&self, directory_path: &PathBuf) -> AppResult<Vec<ResolutionResult>> {
        let start_time = std::time::Instant::now();
        
        let videos = self.file_repo.list_by_type(FileType::Video)?;
        let legenda_files = self.file_repo.list_by_type(FileType::Legenda)?;
        
        let files: Vec<File> = videos
            .into_iter()
            .chain(legenda_files.into_iter())
            .filter(|file| file.caminho_absoluto.starts_with(directory_path))
            .collect();
        
        let total_files = files.len();
        
        let mut results = Vec::with_capacity(total_files);
        let mut resolved_count = 0;
        let mut failed_count = 0;
        let skipped_count = 0;

        for file in files {
            let result = self.resolve_file_internal(&file);
            
            match &result {
                ResolutionResult::Success(_) => resolved_count += 1,
                ResolutionResult::Failure(_) => failed_count += 1,
            }
            
            self.emit_resolution_event(&result);
            results.push(result);
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        self.event_bus.emit(ResolutionBatchCompleted::new(
            total_files,
            resolved_count,
            failed_count,
            skipped_count,
            duration_ms,
        ));

        Ok(results)
    }

    // ========================================================================
    // INTERNAL RESOLUTION LOGIC
    // ========================================================================

    fn resolve_file_internal(&self, file: &File) -> ResolutionResult {
        // Step 1: Determine file role
        let role = match file.tipo {
            FileType::Video => FileRole::Video,
            FileType::Legenda => FileRole::Subtitle,
            FileType::Imagem => FileRole::Image,
            FileType::Outro => {
                return ResolutionResult::Failure(ResolutionFailure::new(
                    file.id,
                    file.caminho_absoluto.clone(),
                    ResolutionFailureReason::UnsupportedFileType,
                    "File type 'outro' is not supported for resolution".to_string(),
                ));
            }
        };

        // Step 2: Parse anime title
        let anime_parse_result = self.rules.parse_anime_title(&file.caminho_absoluto);
        let (anime_title, anime_source) = match anime_parse_result {
            Some((title, source)) => (title, source),
            None => {
                return ResolutionResult::Failure(ResolutionFailure::new(
                    file.id,
                    file.caminho_absoluto.clone(),
                    ResolutionFailureReason::UnparsableTitle,
                    "Could not extract anime title from filename or folder".to_string(),
                ));
            }
        };

        // Step 3: Parse episode number
        let episode_parse_result = self.rules.parse_episode_number(&file.caminho_absoluto);
        let (episode_number, episode_source) = match episode_parse_result {
            Some((num, source)) => (num, source),
            None => {
                return ResolutionResult::Failure(ResolutionFailure::new(
                    file.id,
                    file.caminho_absoluto.clone(),
                    ResolutionFailureReason::UnparsableEpisodeNumber,
                    "Could not extract episode number from filename".to_string(),
                ));
            }
        };

        // Step 4: Try to match against existing anime (READ-ONLY inspection)
        let matched_anime_id = self.try_match_anime(&anime_title).ok().flatten();

        // Step 5: Try to match against existing episode (if anime matched) (READ-ONLY inspection)
        let matched_episode_id = if let Some(anime_id) = matched_anime_id {
            self.try_match_episode(anime_id, &episode_number).ok().flatten()
        } else {
            None
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
            return ResolutionResult::Failure(ResolutionFailure::new(
                file.id,
                file.caminho_absoluto.clone(),
                ResolutionFailureReason::LowConfidence,
                format!(
                    "Confidence {} is below threshold {}",
                    confidence,
                    ResolutionConfidence::THRESHOLD
                ),
            ));
        }

        // Step 8: Build resolved file
        let combined_source = if anime_source == episode_source {
            anime_source
        } else {
            ResolutionSource::Combined
        };

        let anime_intent = if let Some(anime_id) = matched_anime_id {
            ResolvedAnimeIntent::matched(anime_id, anime_title, combined_source.clone())
        } else {
            ResolvedAnimeIntent::from_parsed_title(anime_title, combined_source.clone())
        };

        let episode_intent = if let Some(episode_id) = matched_episode_id {
            ResolvedEpisodeIntent::matched(episode_id, episode_number, combined_source)
        } else {
            ResolvedEpisodeIntent::from_parsed_number(episode_number, combined_source)
        };

        ResolutionResult::Success(ResolvedFile::new(
            file.id,
            file.caminho_absoluto.clone(),
            role,
            anime_intent,
            episode_intent,
            confidence,
        ))
    }

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

    /// Attempt to match parsed title against existing anime entities.
    /// This is a READ-ONLY operation - no entities are created or modified.
    fn try_match_anime(&self, title: &str) -> AppResult<Option<Uuid>> {
        let animes: Vec<Anime> = self.anime_repo.list_all()?;

        // Try exact match first (using titulo_principal)
        if let Some(anime) = animes.iter().find(|a| a.titulo_principal == title) {
            return Ok(Some(anime.id));
        }

        // Try normalized match
        let normalized = self.rules.normalize_title(title);
        if let Some(anime) = animes.iter().find(|a| self.rules.normalize_title(&a.titulo_principal) == normalized) {
            return Ok(Some(anime.id));
        }

        // Try alternative titles
        for anime in &animes {
            for alt_title in &anime.titulos_alternativos {
                if alt_title == title || self.rules.normalize_title(alt_title) == normalized {
                    return Ok(Some(anime.id));
                }
            }
        }

        Ok(None)
    }

    /// Attempt to match parsed episode number against existing episode entities.
    /// This is a READ-ONLY operation - no entities are created or modified.
    fn try_match_episode(
        &self,
        anime_id: Uuid,
        episode_number: &ResolvedEpisodeNumber,
    ) -> AppResult<Option<Uuid>> {
        match episode_number {
            ResolvedEpisodeNumber::Regular { number } => {
                self.episode_repo.find_by_anime_and_number(anime_id, *number)
                    .map(|opt| opt.map(|ep| ep.id))
            }
            ResolvedEpisodeNumber::Special { label } => {
                self.episode_repo.find_by_anime_and_special_label(anime_id, label)
                    .map(|opt| opt.map(|ep| ep.id))
            }
            ResolvedEpisodeNumber::Range { start, .. } => {
                // For ranges, match the first episode
                self.episode_repo.find_by_anime_and_number(anime_id, *start)
                    .map(|opt| opt.map(|ep| ep.id))
            }
        }
    }
}

// ============================================================================
// RESOLUTION RULES (DETERMINISTIC, LAYERED)
// ============================================================================

/// Deterministic rules for parsing and resolving file information.
/// All rules are explicit and ordered.
pub struct ResolutionRules {
    /// Patterns for extracting anime title from filename
    anime_title_patterns: Vec<Regex>,
    
    /// Patterns for extracting episode number from filename
    episode_number_patterns: Vec<Regex>,
    
    /// Patterns for special episode labels
    special_episode_patterns: Vec<Regex>,
}

impl Default for ResolutionRules {
    fn default() -> Self {
        Self {
            anime_title_patterns: vec![
                // [Group] Anime Title - 01 [Quality].mkv
                Regex::new(r"^\[.+?\]\s*(.+?)\s*-\s*\d+").unwrap(),
                // Anime Title - 01.mkv
                Regex::new(r"^(.+?)\s*-\s*\d+").unwrap(),
                // Anime Title S01E01.mkv
                Regex::new(r"^(.+?)\s*S\d+E\d+").unwrap(),
                // Anime Title Episode 01.mkv
                Regex::new(r"^(.+?)\s*[Ee]pisode\s*\d+").unwrap(),
            ],
            episode_number_patterns: vec![
                // - 01, - 001, etc.
                Regex::new(r"-\s*(\d{1,4})(?:\s|\.|\[|$)").unwrap(),
                // S01E01, S1E01
                Regex::new(r"S\d+E(\d+)").unwrap(),
                // Episode 01, Ep 01, EP01
                Regex::new(r"[Ee](?:pisode|p)?\s*(\d+)").unwrap(),
                // #01, #001
                Regex::new(r"#(\d+)").unwrap(),
            ],
            special_episode_patterns: vec![
                // OVA, OVA1, OVA 1
                Regex::new(r"(OVA\s*\d*)").unwrap(),
                // OAD, OAD1
                Regex::new(r"(OAD\s*\d*)").unwrap(),
                // Special, SP, SP1
                Regex::new(r"(Special\s*\d*|SP\s*\d+)").unwrap(),
                // Movie, Film
                Regex::new(r"(Movie|Film)").unwrap(),
            ],
        }
    }
}

impl ResolutionRules {
    /// Parse anime title from file path
    pub fn parse_anime_title(&self, path: &PathBuf) -> Option<(String, ResolutionSource)> {
        // Try filename first
        if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
            for pattern in &self.anime_title_patterns {
                if let Some(captures) = pattern.captures(filename) {
                    if let Some(title) = captures.get(1) {
                        let cleaned = self.clean_title(title.as_str());
                        if !cleaned.is_empty() {
                            return Some((cleaned, ResolutionSource::Filename));
                        }
                    }
                }
            }
        }

        // Try parent folder
        if let Some(parent) = path.parent() {
            if let Some(folder_name) = parent.file_name().and_then(|s| s.to_str()) {
                // Clean up folder name (remove [Group], quality tags, etc.)
                let cleaned = self.clean_folder_name(folder_name);
                if !cleaned.is_empty() && cleaned.len() > 2 {
                    return Some((cleaned, ResolutionSource::FolderName));
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

        // Bonus for filename-based resolution (most reliable)
        if *anime_source == ResolutionSource::Filename {
            score += 0.1;
        }
        if *episode_source == ResolutionSource::Filename {
            score += 0.1;
        }

        // Penalty for short titles (likely incorrect)
        if anime_title.len() < 3 {
            score -= 0.2;
        }

        // Bonus for regular episode numbers (more common)
        if matches!(episode_number, ResolvedEpisodeNumber::Regular { .. }) {
            score += 0.05;
        }

        ResolutionConfidence::new(score)
    }

    /// Normalize title for matching
    pub fn normalize_title(&self, title: &str) -> String {
        title
            .to_lowercase()
            .replace([':', ';', '!', '?', '.', ','], "")
            .replace(['_', '-'], " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Clean up extracted title
    fn clean_title(&self, title: &str) -> String {
        let mut cleaned = title.trim().to_string();
        
        // Remove trailing quality tags
        let quality_patterns = [
            r"\s*\[.*?\]\s*$",
            r"\s*\(.*?\)\s*$",
            r"\s*1080p\s*$",
            r"\s*720p\s*$",
            r"\s*480p\s*$",
            r"\s*HEVC\s*$",
            r"\s*x264\s*$",
            r"\s*x265\s*$",
        ];

        for pattern in quality_patterns {
            if let Ok(re) = Regex::new(pattern) {
                cleaned = re.replace_all(&cleaned, "").to_string();
            }
        }

        cleaned.trim().to_string()
    }

    /// Clean up folder name for title extraction
    fn clean_folder_name(&self, folder: &str) -> String {
        let mut cleaned = folder.to_string();

        // Remove leading [Group] tags
        if let Ok(re) = Regex::new(r"^\[.+?\]\s*") {
            cleaned = re.replace(&cleaned, "").to_string();
        }

        // Remove trailing quality/season tags
        let patterns = [
            r"\s*\[.*?\]\s*$",
            r"\s*\(.*?\)\s*$",
            r"\s*S\d+\s*$",
            r"\s*Season\s*\d+\s*$",
        ];

        for pattern in patterns {
            if let Ok(re) = Regex::new(pattern) {
                cleaned = re.replace_all(&cleaned, "").to_string();
            }
        }

        cleaned.trim().to_string()
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
            &ResolvedEpisodeNumber::Special { label: "?".to_string() },
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

        assert_eq!(
            rules.normalize_title("Steins;Gate"),
            "steinsgate"
        );
        assert_eq!(
            rules.normalize_title("Attack_on_Titan"),
            "attack on titan"
        );
        assert_eq!(
            rules.normalize_title("Re:Zero"),
            "rezero"
        );
    }
}
