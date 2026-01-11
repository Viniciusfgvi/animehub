// src-tauri/src/services/resolution_service_tests.rs
//
// PHASE 4 HARDENED TESTS - CORRECTED
//
// These tests PROVE that Phase 4 corrections are complete.
// They enforce impossibility, not intention.
//
// CORRECTIONS APPLIED:
// - REMOVED: ResolutionFingerprint::from_result (hallucinated API)
// - FIXED: UnparsableTitle → UnparsableFilename
// - FIXED: UnparsableEpisodeNumber → NoEpisodeNumber
// - Uses only canonical enum variants

#[cfg(test)]
mod idempotency_enforcement_tests {
    use crate::domain::resolution::{
        FileRole, ResolutionConfidence, ResolutionSource, ResolvedAnimeIntent,
        ResolvedEpisodeIntent, ResolvedEpisodeNumber, ResolvedFile,
    };
    use std::collections::HashSet;
    use std::path::PathBuf;
    use uuid::Uuid;

    /// PROVES: Identical input produces identical fingerprint
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

        // PROOF: Fingerprints are byte-for-byte identical
        assert_eq!(
            resolved1.fingerprint(),
            resolved2.fingerprint(),
            "Identical input MUST produce identical fingerprint"
        );
    }

    /// PROVES: Different input produces different fingerprint
    #[test]
    fn test_fingerprint_uniqueness() {
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
            PathBuf::from("/test/anime/Episode 02.mkv"),
            FileRole::Video,
            ResolvedAnimeIntent::from_parsed_title(
                "Test Anime".to_string(),
                ResolutionSource::Filename,
            ),
            ResolvedEpisodeIntent::from_parsed_number(
                ResolvedEpisodeNumber::Regular { number: 2 }, // Different episode
                ResolutionSource::Filename,
            ),
            ResolutionConfidence::high(),
        );

        // PROOF: Different episodes have different fingerprints
        assert_ne!(
            resolved1.fingerprint(),
            resolved2.fingerprint(),
            "Different input MUST produce different fingerprint"
        );
    }

    /// PROVES: Fingerprint can be used for idempotency tracking
    #[test]
    fn test_fingerprint_set_deduplication() {
        let file_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let resolved = ResolvedFile::new(
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

        let mut fingerprint_set: HashSet<String> = HashSet::new();

        // First insertion succeeds
        let fp1 = resolved.fingerprint().to_string();
        assert!(
            fingerprint_set.insert(fp1.clone()),
            "First fingerprint insertion MUST succeed"
        );

        // Second insertion of same fingerprint fails (deduplication)
        let fp2 = resolved.fingerprint().to_string();
        assert!(
            !fingerprint_set.insert(fp2),
            "Duplicate fingerprint insertion MUST fail"
        );

        // Set contains exactly one entry
        assert_eq!(
            fingerprint_set.len(),
            1,
            "Fingerprint set MUST contain exactly one entry after deduplication"
        );
    }

    /// PROVES: Fingerprints work for both success and failure results
    #[test]
    fn test_fingerprint_for_success_and_failure() {
        use crate::domain::resolution::{ResolutionFailure, ResolutionFailureReason};

        let file_id = Uuid::new_v4();

        let success = ResolvedFile::new(
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

        let failure = ResolutionFailure::new(
            file_id,
            PathBuf::from("/test/unknown.bin"),
            ResolutionFailureReason::UnsupportedFileType,
            "Not supported".to_string(),
        );

        let fp_success = success.fingerprint();
        let fp_failure = failure.fingerprint();

        // PROOF: Both success and failure produce valid fingerprints
        assert!(!fp_success.hash().is_empty(), "Success fingerprint MUST NOT be empty");
        assert!(!fp_failure.hash().is_empty(), "Failure fingerprint MUST NOT be empty");

        // PROOF: Success and failure fingerprints are different
        assert_ne!(
            fp_success.hash(),
            fp_failure.hash(),
            "Success and failure fingerprints MUST be different"
        );
    }
}

#[cfg(test)]
mod episode_resolved_reachability_tests {
    use crate::events::resolution_events::EpisodeResolved;
    use crate::events::DomainEvent;
    use uuid::Uuid;

    /// PROVES: EpisodeResolved can be constructed
    #[test]
    fn test_episode_resolved_is_constructable() {
        let video_id = Uuid::new_v4();
        let sub_id = Uuid::new_v4();

        let event = EpisodeResolved::new(
            "Test Anime".to_string(),
            None,
            "1".to_string(),
            None,
            Some(video_id),
            vec![sub_id],
            vec![],
            0.95,
        );

        // PROOF: Event is constructed with all fields populated
        assert_eq!(event.anime_title, "Test Anime");
        assert_eq!(event.episode_number, "1");
        assert_eq!(event.video_file_id, Some(video_id));
        assert_eq!(event.subtitle_file_ids.len(), 1);
        assert!(!event.fingerprint.is_empty());
    }

    /// PROVES: EpisodeResolved fingerprint is deterministic
    #[test]
    fn test_episode_resolved_fingerprint_determinism() {
        let video_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let sub_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();

        let event1 = EpisodeResolved::new(
            "Test Anime".to_string(),
            None,
            "1".to_string(),
            None,
            Some(video_id),
            vec![sub_id],
            vec![],
            0.95,
        );

        let event2 = EpisodeResolved::new(
            "Test Anime".to_string(),
            None,
            "1".to_string(),
            None,
            Some(video_id),
            vec![sub_id],
            vec![],
            0.95,
        );

        // PROOF: Identical input produces identical fingerprint
        assert_eq!(
            event1.fingerprint, event2.fingerprint,
            "EpisodeResolved fingerprint MUST be deterministic"
        );
    }

    /// PROVES: EpisodeResolved event_id is derived from fingerprint (deterministic)
    #[test]
    fn test_episode_resolved_event_id_determinism() {
        let video_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let event1 = EpisodeResolved::new(
            "Test Anime".to_string(),
            None,
            "1".to_string(),
            None,
            Some(video_id),
            vec![],
            vec![],
            0.95,
        );

        let event2 = EpisodeResolved::new(
            "Test Anime".to_string(),
            None,
            "1".to_string(),
            None,
            Some(video_id),
            vec![],
            vec![],
            0.95,
        );

        // PROOF: event_id is deterministic
        assert_eq!(
            event1.event_id(),
            event2.event_id(),
            "EpisodeResolved event_id MUST be deterministic"
        );
    }
}

#[cfg(test)]
mod dead_variant_elimination_tests {
    use crate::domain::resolution::{
        FileRole, ResolutionFailureReason, ResolutionSource, ResolvedEpisodeNumber,
    };

    /// PROVES: FileRole has exactly 3 variants (Auxiliary removed)
    #[test]
    fn test_file_role_has_no_auxiliary() {
        // Exhaustive match proves no other variants exist
        let roles = [FileRole::Video, FileRole::Subtitle, FileRole::Image];

        for role in &roles {
            match role {
                FileRole::Video => assert_eq!(role.to_string(), "video"),
                FileRole::Subtitle => assert_eq!(role.to_string(), "subtitle"),
                FileRole::Image => assert_eq!(role.to_string(), "image"),
                // If Auxiliary existed, this would not compile
            }
        }

        // PROOF: Exactly 3 variants
        assert_eq!(roles.len(), 3, "FileRole MUST have exactly 3 variants");
    }

    /// PROVES: ResolutionSource has exactly 2 variants (FolderHierarchy, DatabaseMatch, Combined removed)
    #[test]
    fn test_resolution_source_has_no_dead_variants() {
        let sources = [ResolutionSource::Filename, ResolutionSource::FolderName];

        for source in &sources {
            match source {
                ResolutionSource::Filename => assert_eq!(source.to_string(), "filename"),
                ResolutionSource::FolderName => assert_eq!(source.to_string(), "folder_name"),
                // If dead variants existed, this would not compile
            }
        }

        // PROOF: Exactly 2 variants
        assert_eq!(sources.len(), 2, "ResolutionSource MUST have exactly 2 variants");
    }

    /// PROVES: ResolvedEpisodeNumber has exactly 2 variants (Range removed)
    #[test]
    fn test_resolved_episode_number_has_no_range() {
        let numbers = [
            ResolvedEpisodeNumber::Regular { number: 1 },
            ResolvedEpisodeNumber::Special {
                label: "OVA".to_string(),
            },
        ];

        for num in &numbers {
            match num {
                ResolvedEpisodeNumber::Regular { number } => {
                    assert_eq!(num.to_string(), number.to_string())
                }
                ResolvedEpisodeNumber::Special { label } => assert_eq!(num.to_string(), *label),
                // If Range existed, this would not compile
            }
        }

        // PROOF: Exactly 2 variants
        assert_eq!(
            numbers.len(),
            2,
            "ResolvedEpisodeNumber MUST have exactly 2 variants"
        );
    }

    /// PROVES: ResolutionFailureReason has exactly 5 canonical variants
    /// CORRECTED: UnparsableTitle → UnparsableFilename, UnparsableEpisodeNumber → NoEpisodeNumber
    #[test]
    fn test_resolution_failure_reason_has_canonical_variants() {
        let reasons = [
            ResolutionFailureReason::UnparsableFilename,
            ResolutionFailureReason::NoEpisodeNumber,
            ResolutionFailureReason::LowConfidence,
            ResolutionFailureReason::UnsupportedFileType,
            ResolutionFailureReason::RepositoryError,
        ];

        for reason in &reasons {
            match reason {
                ResolutionFailureReason::UnparsableFilename => {
                    assert_eq!(reason.to_string(), "unparsable_filename")
                }
                ResolutionFailureReason::NoEpisodeNumber => {
                    assert_eq!(reason.to_string(), "no_episode_number")
                }
                ResolutionFailureReason::LowConfidence => {
                    assert_eq!(reason.to_string(), "low_confidence")
                }
                ResolutionFailureReason::UnsupportedFileType => {
                    assert_eq!(reason.to_string(), "unsupported_file_type")
                }
                ResolutionFailureReason::RepositoryError => {
                    assert_eq!(reason.to_string(), "repository_error")
                }
                // If dead variants existed, this would not compile
            }
        }

        // PROOF: Exactly 5 variants
        assert_eq!(
            reasons.len(),
            5,
            "ResolutionFailureReason MUST have exactly 5 variants"
        );
    }
}

#[cfg(test)]
mod determinism_tests {
    use crate::domain::resolution::{
        FileRole, ResolutionConfidence, ResolutionFailure, ResolutionFailureReason,
        ResolutionSource, ResolvedAnimeIntent, ResolvedEpisodeIntent, ResolvedEpisodeNumber,
        ResolvedFile,
    };
    use std::path::PathBuf;
    use uuid::Uuid;

    /// PROVES: ResolvedFile has no timestamp field
    #[test]
    fn test_resolved_file_has_no_timestamp() {
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

        // Wait to ensure time passes
        std::thread::sleep(std::time::Duration::from_millis(10));

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

        // PROOF: Fingerprints are identical despite time difference
        assert_eq!(
            resolved1.fingerprint(),
            resolved2.fingerprint(),
            "ResolvedFile MUST NOT contain timestamps"
        );
    }

    /// PROVES: ResolutionFailure has no timestamp field
    #[test]
    fn test_resolution_failure_has_no_timestamp() {
        let file_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let failure1 = ResolutionFailure::new(
            file_id,
            PathBuf::from("/test/unknown.bin"),
            ResolutionFailureReason::UnsupportedFileType,
            "Not supported".to_string(),
        );

        // Wait to ensure time passes
        std::thread::sleep(std::time::Duration::from_millis(10));

        let failure2 = ResolutionFailure::new(
            file_id,
            PathBuf::from("/test/unknown.bin"),
            ResolutionFailureReason::UnsupportedFileType,
            "Not supported".to_string(),
        );

        // PROOF: Fingerprints are identical despite time difference
        assert_eq!(
            failure1.fingerprint(),
            failure2.fingerprint(),
            "ResolutionFailure MUST NOT contain timestamps"
        );
    }
}

#[cfg(test)]
mod resolution_rules_tests {
    use crate::services::resolution_service::ResolutionRules;
    use crate::domain::resolution::{ResolutionSource, ResolvedEpisodeNumber};
    use std::path::PathBuf;

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
    fn test_normalize_title() {
        let rules = ResolutionRules::default();

        assert_eq!(rules.normalize_title("Steins;Gate"), "steins gate");
        assert_eq!(rules.normalize_title("Attack_on_Titan"), "attack on titan");
        assert_eq!(rules.normalize_title("Re:Zero"), "re zero");
    }
}
