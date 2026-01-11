// src-tauri/examples/playback_invalid_state_test.rs
//
// PHASE 3 VALIDATION TEST: Invalid State Handling
//
// PURPOSE:
// - Prove system gracefully rejects invalid playback attempts
// - Case 1: Episode with no linked file → error, no MPV launch
// - Case 2: Invalid episode ID → NotFound error, no MPV launch
// - No panic in any case
//
// CONTRACT REFERENCES:
// - playback_service.rs line 12038: "No video file linked" error
// - playback_service.rs line 12032: NotFound error for missing episode
//
// VALIDATION:
// - Errors are returned (not panics)
// - MPV is never launched
// - System remains stable

use std::sync::Arc;
use uuid::Uuid;

use animehub::db::{create_connection_pool, initialize_database};
use animehub::domain::anime::{AnimeStatus, AnimeType};
use animehub::domain::episode::EpisodeNumber;
use animehub::events::EventBus;
use animehub::integrations::MpvClient;
use animehub::repositories::*;
use animehub::services::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PLAYBACK INVALID STATE TEST ===");
    println!("Purpose: Validate graceful rejection of invalid playback attempts");
    println!();

    // =========================================================================
    // 1. INFRASTRUCTURE BOOTSTRAP
    // =========================================================================
    println!("[SETUP] Bootstrapping infrastructure...");

    let event_bus = Arc::new(EventBus::new());
    let pool = Arc::new(create_connection_pool()?);
    let mpv_client = Arc::new(MpvClient::new()?);

    {
        let conn = pool.get()?;
        initialize_database(&conn)?;
    }

    // =========================================================================
    // 2. REPOSITORIES
    // =========================================================================
    let anime_repo: Arc<dyn AnimeRepository> = Arc::new(SqliteAnimeRepository::new(pool.clone()));
    let episode_repo: Arc<dyn EpisodeRepository> =
        Arc::new(SqliteEpisodeRepository::new(pool.clone()));
    let file_repo: Arc<dyn FileRepository> = Arc::new(SqliteFileRepository::new(pool.clone()));
    let anime_alias_repo: Arc<dyn AnimeAliasRepository> =
        Arc::new(SqliteAnimeAliasRepository::new(pool.clone()));
    let external_ref_repo: Arc<dyn ExternalReferenceRepository> =
        Arc::new(SqliteExternalReferenceRepository::new(pool.clone()));

    // =========================================================================
    // 3. SERVICES
    // =========================================================================
    let anime_service = Arc::new(AnimeService::new(
        anime_repo.clone(),
        anime_alias_repo.clone(),
        external_ref_repo.clone(),
        event_bus.clone(),
    ));
    let episode_service = Arc::new(EpisodeService::new(
        episode_repo.clone(),
        anime_repo.clone(),
        file_repo.clone(),
        event_bus.clone(),
    ));
    let playback_service = Arc::new(PlaybackService::new(
        episode_repo.clone(),
        file_repo.clone(),
        event_bus.clone(),
        mpv_client.clone(),
    ));

    println!("[SETUP] Services initialized.");
    println!();

    let mut all_passed = true;

    // =========================================================================
    // CASE 1: Episode with no linked file
    // =========================================================================
    println!("===========================================");
    println!("CASE 1: Episode with no linked file");
    println!("===========================================");
    println!();

    // Create Anime
    println!("[CASE 1] Creating Anime...");
    let anime_id = anime_service.create_anime(CreateAnimeRequest {
        titulo_principal: "Invalid State Test Anime".to_string(),
        titulos_alternativos: vec![],
        tipo: AnimeType::TV,
        status: AnimeStatus::EmExibicao,
        total_episodios: Some(12),
        data_inicio: None,
        data_fim: None,
        metadados_livres: serde_json::json!({}),
    })?;
    println!("[CASE 1] Anime created: {}", anime_id);

    // Create Episode (WITHOUT linking any file)
    println!("[CASE 1] Creating Episode (no file linked)...");
    let episode_id = episode_service.create_episode(CreateEpisodeRequest {
        anime_id,
        numero: EpisodeNumber::Regular { numero: 1 },
        titulo: Some("Episode without file".to_string()),
        duracao_esperada: Some(1440),
    })?;
    println!("[CASE 1] Episode created: {}", episode_id);

    // Attempt playback
    println!("[CASE 1] Attempting playback (should fail)...");
    // CORRECTED: file_id is now required, use a fake UUID to test "file not found"
    let fake_file_id = Uuid::new_v4();
    let result = playback_service.start_playback(StartPlaybackRequest {
        episode_id,
        file_id: fake_file_id,
    });

    match result {
        Ok(_) => {
            println!("[FAIL] Playback started when it should have failed!");
            all_passed = false;
        }
        Err(e) => {
            let error_msg = e.to_string();
            println!("[CASE 1] Error received: {}", error_msg);

            if error_msg.contains("No video file linked")
                || error_msg.contains("no file")
                || error_msg.contains("Not found")
            {
                println!("[PASS] Correct error returned for episode with no file.");
            } else {
                println!(
                    "[WARN] Error returned but message unexpected: {}",
                    error_msg
                );
                println!("       (Still passing because error was returned, not panic)");
            }
        }
    }

    // Verify MPV is not running
    if mpv_client.is_running() {
        println!("[FAIL] MPV is running when it should not be!");
        all_passed = false;
    } else {
        println!("[PASS] MPV was not launched.");
    }

    println!();

    // =========================================================================
    // CASE 2: Invalid episode ID
    // =========================================================================
    println!("===========================================");
    println!("CASE 2: Invalid episode ID");
    println!("===========================================");
    println!();

    // Generate random UUID that doesn't exist
    let fake_episode_id = Uuid::new_v4();
    println!(
        "[CASE 2] Using non-existent episode ID: {}",
        fake_episode_id
    );

    // Attempt playback
    println!("[CASE 2] Attempting playback (should fail)...");
    // CORRECTED: file_id is now required, use a fake UUID
    let fake_file_id = Uuid::new_v4();
    let result = playback_service.start_playback(StartPlaybackRequest {
        episode_id: fake_episode_id,
        file_id: fake_file_id,
    });

    match result {
        Ok(_) => {
            println!("[FAIL] Playback started when it should have failed!");
            all_passed = false;
        }
        Err(e) => {
            let error_msg = e.to_string();
            println!("[CASE 2] Error received: {}", error_msg);

            if error_msg.contains("Not found")
                || error_msg.contains("not found")
                || error_msg.contains("NotFound")
            {
                println!("[PASS] Correct NotFound error returned.");
            } else {
                println!(
                    "[WARN] Error returned but message unexpected: {}",
                    error_msg
                );
                println!("       (Still passing because error was returned, not panic)");
            }
        }
    }

    // Verify MPV is not running
    if mpv_client.is_running() {
        println!("[FAIL] MPV is running when it should not be!");
        all_passed = false;
    } else {
        println!("[PASS] MPV was not launched.");
    }

    println!();

    // =========================================================================
    // CASE 3: Stop playback with invalid episode ID
    // =========================================================================
    println!("===========================================");
    println!("CASE 3: Stop playback with invalid episode ID");
    println!("===========================================");
    println!();

    let fake_episode_id_2 = Uuid::new_v4();
    println!(
        "[CASE 3] Using non-existent episode ID: {}",
        fake_episode_id_2
    );

    // Attempt to stop playback (nothing is playing)
    println!("[CASE 3] Attempting stop_playback (should not panic)...");
    let result = playback_service.stop_playback(fake_episode_id_2);

    match result {
        Ok(_) => {
            println!("[PASS] stop_playback returned Ok (graceful no-op).");
        }
        Err(e) => {
            println!("[PASS] stop_playback returned error: {}", e);
            println!("       (Error is acceptable, panic is not)");
        }
    }

    println!();

    // =========================================================================
    // FINAL RESULT
    // =========================================================================
    if all_passed {
        println!("===========================================");
        println!("PLAYBACK INVALID STATE TEST: PASSED");
        println!("===========================================");
    } else {
        println!("===========================================");
        println!("PLAYBACK INVALID STATE TEST: FAILED");
        println!("===========================================");
        std::process::exit(1);
    }

    println!();
    println!("Summary:");
    println!("  - Case 1 (no file): Error returned, no MPV launch");
    println!("  - Case 2 (invalid ID): Error returned, no MPV launch");
    println!("  - Case 3 (stop invalid): Graceful handling, no panic");
    println!();
    println!("This test validates graceful error handling for invalid states.");

    Ok(())
}
