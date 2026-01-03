// src-tauri/examples/scan_validation_test.rs
//
// PHASE 3 VALIDATION TEST: Pre-Resolution Phase (Scan)
//
// PURPOSE:
// - Validate that directory scan detects files correctly
// - Validate correct event emission (FileDetected, DirectoryScanned)
// - Validate that NO domain entities (Anime, Episode) are implicitly created
//
// CONTRACT REFERENCES:
// - SERVICE_CONTRACTS.md Section 3: File Service
// - EVENT_MAP.md Section 1: DirectoryScanned
// - EVENT_MAP.md Section 2: FileDetected
//
// DOES NOT:
// - Start playback
// - Infer Anime or Episode
// - Create domain entities

use std::sync::Arc;
use std::path::PathBuf;

use animehub::db::{create_connection_pool, initialize_database};
use animehub::events::EventBus;
use animehub::repositories::*;
use animehub::services::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SCAN VALIDATION TEST ===");
    println!("Purpose: Validate pre-resolution phase (scan only)");
    println!();

    // =========================================================================
    // 1. INFRASTRUCTURE BOOTSTRAP (same as main.rs)
    // =========================================================================
    println!("[SETUP] Bootstrapping infrastructure...");
    
    let event_bus = Arc::new(EventBus::new());
    let pool = Arc::new(create_connection_pool()?);

    // Initialize schema (idempotent)
    {
        let conn = pool.get()?;
        initialize_database(&conn)?;
    }
    println!("[SETUP] Database initialized.");

    // =========================================================================
    // 2. REPOSITORIES
    // =========================================================================
    let anime_repo: Arc<dyn AnimeRepository> = Arc::new(SqliteAnimeRepository::new(pool.clone()));
    let episode_repo: Arc<dyn EpisodeRepository> = Arc::new(SqliteEpisodeRepository::new(pool.clone()));
    let file_repo: Arc<dyn FileRepository> = Arc::new(SqliteFileRepository::new(pool.clone()));

    // =========================================================================
    // 3. SERVICES (only FileService needed for this test)
    // =========================================================================
    let file_service = Arc::new(FileService::new(file_repo.clone(), event_bus.clone()));

    println!("[SETUP] Services initialized.");
    println!();

    // =========================================================================
    // 4. GET TEST DIRECTORY FROM ARGS
    // =========================================================================
    let test_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example scan_validation_test -- <directory_path>");
            eprintln!("       The directory should contain at least one video file.");
            std::process::exit(1);
        });

    if !test_dir.exists() {
        eprintln!("[ERROR] Directory does not exist: {:?}", test_dir);
        std::process::exit(1);
    }

    println!("[TEST] Scanning directory: {:?}", test_dir);
    println!();

    // =========================================================================
    // 5. CLEAR EVENT LOG (for clean observation)
    // =========================================================================
    event_bus.clear_event_log();

    // =========================================================================
    // 6. COUNT ENTITIES BEFORE SCAN
    // =========================================================================
    let animes_before = anime_repo.list_all()?.len();
    let episodes_before: usize = {
        // Count all episodes across all animes
        let animes = anime_repo.list_all()?;
        animes.iter()
            .map(|a| episode_repo.list_by_anime(a.id).map(|e| e.len()).unwrap_or(0))
            .sum()
    };
    
    println!("[PRE-SCAN] Anime count: {}", animes_before);
    println!("[PRE-SCAN] Episode count: {}", episodes_before);
    println!();

    // =========================================================================
    // 6.5 REGISTER EVENT OBSERVERS (must happen BEFORE scan)
    // =========================================================================
    event_bus.subscribe::<animehub::events::FileDetected, _>(|_event| {
        // no-op: ensures event is observed and logged
    });
    
    event_bus.subscribe::<animehub::events::DirectoryScanned, _>(|_event| {
        // no-op: ensures event is observed and logged
    });


    // =========================================================================
    // 7. EXECUTE SCAN
    // =========================================================================
    println!("[SCAN] Starting directory scan...");
    println!("--- Events will be printed below ---");
    println!();

    let files_found = file_service.scan_directory(test_dir.clone())?;

    println!();
    println!("--- End of events ---");
    println!();
    println!("[SCAN] Files detected: {}", files_found);
    println!();

    // =========================================================================
    // 8. COUNT ENTITIES AFTER SCAN
    // =========================================================================
    let animes_after = anime_repo.list_all()?.len();
    let episodes_after: usize = {
        let animes = anime_repo.list_all()?;
        animes.iter()
            .map(|a| episode_repo.list_by_anime(a.id).map(|e| e.len()).unwrap_or(0))
            .sum()
    };

    println!("[POST-SCAN] Anime count: {}", animes_after);
    println!("[POST-SCAN] Episode count: {}", episodes_after);
    println!();

    // =========================================================================
    // 9. VALIDATE: NO IMPLICIT ENTITY CREATION
    // =========================================================================
    println!("[VALIDATION] Checking for implicit entity creation...");
    
    if animes_after != animes_before {
        println!("[FAIL] Anime entities were implicitly created!");
        println!("       Before: {}, After: {}", animes_before, animes_after);
        std::process::exit(1);
    }
    
    if episodes_after != episodes_before {
        println!("[FAIL] Episode entities were implicitly created!");
        println!("       Before: {}, After: {}", episodes_before, episodes_after);
        std::process::exit(1);
    }

    println!("[PASS] No implicit Anime or Episode entities created.");
    println!();

    // =========================================================================
    // 10. VALIDATE: EVENT LOG
    // =========================================================================
    println!("[VALIDATION] Checking event log...");
    
    let event_log = event_bus.get_event_log();
    
    let file_detected_count = event_log.iter()
        .filter(|e| e.event_type == "FileDetected")
        .count();
    
    let directory_scanned_count = event_log.iter()
        .filter(|e| e.event_type == "DirectoryScanned")
        .count();

    println!("  FileDetected events: {}", file_detected_count);
    println!("  DirectoryScanned events: {}", directory_scanned_count);

    if files_found > 0 && file_detected_count == 0 {
        println!("[FAIL] Files were found but no FileDetected events emitted!");
        std::process::exit(1);
    }

    if directory_scanned_count != 1 {
        println!("[FAIL] Expected exactly 1 DirectoryScanned event, got {}", directory_scanned_count);
        std::process::exit(1);
    }

    println!("[PASS] Events emitted correctly.");
    println!();

    // =========================================================================
    // 11. FINAL RESULT
    // =========================================================================
    println!("===========================================");
    println!("SCAN VALIDATION TEST: PASSED");
    println!("===========================================");
    println!();
    println!("Summary:");
    println!("  - Directory scanned: {:?}", test_dir);
    println!("  - Files detected: {}", files_found);
    println!("  - FileDetected events: {}", file_detected_count);
    println!("  - DirectoryScanned events: {}", directory_scanned_count);
    println!("  - Implicit Anime created: 0 (correct)");
    println!("  - Implicit Episode created: 0 (correct)");
    println!();
    println!("This test seals the pre-resolution phase of the pipeline.");

    Ok(())
}
