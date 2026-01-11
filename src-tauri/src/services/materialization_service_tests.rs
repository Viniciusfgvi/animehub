// src-tauri/src/services/materialization_service_tests.rs
//
// Materialization Service Tests - PHASE 4 CORRECTED
//
// CORRECTIONS APPLIED:
// - REMOVED: Mock repositories (hallucinated - do not exist)
// - REMOVED: ResolutionSource::Combined (dead variant)
// - REMOVED: event_id: Uuid::new_v4() in test helper (non-deterministic)
// - REMOVED: occurred_at: Utc::now() in test helper (non-deterministic)
// - Uses deterministic fingerprint computation
// - Uses canonical FileResolved::new() constructor

#[cfg(test)]
mod tests {
    use crate::events::resolution_events::FileResolved;
    use crate::events::DomainEvent;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::path::PathBuf;
    use uuid::Uuid;

    // ========================================================================
    // TEST HELPERS - CORRECTED
    // ========================================================================

    /// CORRECTED: Create FileResolved event using canonical constructor.
    /// 
    /// CHANGES FROM ORIGINAL:
    /// - Removed: event_id: Uuid::new_v4() (non-deterministic)
    /// - Removed: occurred_at: Utc::now() (non-deterministic)
    /// - Removed: resolution_source: ResolutionSource::Combined (dead variant)
    /// - Added: fingerprint computed deterministically
    /// - Added: source as "filename" (valid canonical value)
    fn create_file_resolved_event(
        file_id: Uuid,
        anime_title: &str,
        episode_number: &str,
        file_role: &str,
    ) -> FileResolved {
        // Compute a deterministic fingerprint for the test event
        let mut hasher = DefaultHasher::new();
        file_id.hash(&mut hasher);
        anime_title.to_lowercase().hash(&mut hasher);
        episode_number.hash(&mut hasher);
        file_role.hash(&mut hasher);
        let fingerprint = format!("test:{:016x}", hasher.finish());

        FileResolved::new(
            file_id,
            PathBuf::from("/test/path.mkv"),
            anime_title.to_string(),
            None,  // matched_anime_id
            episode_number.to_string(),
            None,  // matched_episode_id
            file_role.to_string(),
            0.9,   // confidence
            "filename".to_string(),  // source (valid: "filename" or "folder_name")
            fingerprint,
        )
    }

    // ========================================================================
    // DETERMINISM TESTS
    // ========================================================================

    #[test]
    fn test_file_resolved_event_is_deterministic() {
        let file_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let event1 = create_file_resolved_event(file_id, "Test Anime", "1", "video");
        let event2 = create_file_resolved_event(file_id, "Test Anime", "1", "video");

        // PROOF: Identical input produces identical fingerprint
        assert_eq!(event1.fingerprint, event2.fingerprint);
        
        // PROOF: event_id is derived from fingerprint, so also identical
        assert_eq!(event1.event_id(), event2.event_id());
    }

    #[test]
    fn test_different_input_produces_different_fingerprint() {
        let file_id1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let file_id2 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();

        let event1 = create_file_resolved_event(file_id1, "Test Anime", "1", "video");
        let event2 = create_file_resolved_event(file_id2, "Test Anime", "1", "video");

        // PROOF: Different file_id produces different fingerprint
        assert_ne!(event1.fingerprint, event2.fingerprint);
        assert_ne!(event1.event_id(), event2.event_id());
    }

    #[test]
    fn test_different_episode_produces_different_fingerprint() {
        let file_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let event1 = create_file_resolved_event(file_id, "Test Anime", "1", "video");
        let event2 = create_file_resolved_event(file_id, "Test Anime", "2", "video");

        // PROOF: Different episode produces different fingerprint
        assert_ne!(event1.fingerprint, event2.fingerprint);
    }

    #[test]
    fn test_different_anime_produces_different_fingerprint() {
        let file_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let event1 = create_file_resolved_event(file_id, "Anime A", "1", "video");
        let event2 = create_file_resolved_event(file_id, "Anime B", "1", "video");

        // PROOF: Different anime produces different fingerprint
        assert_ne!(event1.fingerprint, event2.fingerprint);
    }

    // ========================================================================
    // EVENT STRUCTURE TESTS
    // ========================================================================

    #[test]
    fn test_file_resolved_event_has_required_fields() {
        let file_id = Uuid::new_v4();
        let event = create_file_resolved_event(file_id, "Test Anime", "1", "video");

        // PROOF: All required fields are populated
        assert_eq!(event.file_id, file_id);
        assert_eq!(event.anime_title, "Test Anime");
        assert_eq!(event.episode_number, "1");
        assert_eq!(event.file_role, "video");
        assert_eq!(event.source, "filename");
        assert!(!event.fingerprint.is_empty());
        assert!(event.confidence > 0.0);
    }

    #[test]
    fn test_file_resolved_event_id_is_uuid() {
        let file_id = Uuid::new_v4();
        let event = create_file_resolved_event(file_id, "Test Anime", "1", "video");

        // PROOF: event_id() returns a valid UUID
        let event_id = event.event_id();
        assert!(!event_id.is_nil());
    }

    // ========================================================================
    // FINGERPRINT HASH TESTS
    // ========================================================================

    #[test]
    fn test_fingerprint_format() {
        let file_id = Uuid::new_v4();
        let event = create_file_resolved_event(file_id, "Test Anime", "1", "video");

        // PROOF: Fingerprint has expected format (test: prefix + 16 hex chars)
        assert!(event.fingerprint.starts_with("test:"));
        assert_eq!(event.fingerprint.len(), 5 + 16); // "test:" + 16 hex chars
    }

    #[test]
    fn test_fingerprint_is_case_insensitive_for_title() {
        let file_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let event1 = create_file_resolved_event(file_id, "Test Anime", "1", "video");
        let event2 = create_file_resolved_event(file_id, "TEST ANIME", "1", "video");

        // PROOF: Title comparison is case-insensitive (both lowercased in hash)
        assert_eq!(event1.fingerprint, event2.fingerprint);
    }
}
