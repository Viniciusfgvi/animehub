use super::entity::Episode;
use crate::domain::{DomainError, DomainResult};

/// Validates all Episode invariants
pub fn validate_episode(episode: &Episode) -> DomainResult<()> {
    validate_progress(episode)?;
    Ok(())
}

/// Progress invariants:
/// 1. Progress cannot be negative (enforced by u64)
/// 2. Progress cannot exceed duration (if known)
fn validate_progress(episode: &Episode) -> DomainResult<()> {
    if let Some(duracao) = episode.duracao_esperada {
        if episode.progresso_atual > duracao {
            return Err(DomainError::ProgressExceedsDuration {
                progress: episode.progresso_atual,
                duration: duracao,
            });
        }
    }
    Ok(())
}

/// Critical Episode Invariants:
/// 
/// 1. Episode MUST belong to exactly one Anime (anime_id required)
/// 2. Episode can exist without a file
/// 3. Episode assumes one practical version (no implicit multi-version)
/// 4. Progress never decreases automatically
/// 5. Progress never exceeds duration (if known)
/// 6. State transitions are explicit
/// 7. Episode ID is immutable
/// 8. anime_id is immutable (episode cannot change parent)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::episode::{Episode, EpisodeNumber};
    use uuid::Uuid;

    #[test]
    fn test_valid_episode() {
        let anime_id = Uuid::new_v4();
        let episode = Episode::new(anime_id, EpisodeNumber::regular(1));
        assert!(validate_episode(&episode).is_ok());
    }

    #[test]
    fn test_progress_within_duration() {
        let anime_id = Uuid::new_v4();
        let mut episode = Episode::new(anime_id, EpisodeNumber::regular(1));
        episode.duracao_esperada = Some(1440); // 24 minutes
        episode.progresso_atual = 1200; // 20 minutes
        assert!(validate_episode(&episode).is_ok());
    }

    #[test]
    fn test_progress_exceeds_duration_fails() {
        let anime_id = Uuid::new_v4();
        let mut episode = Episode::new(anime_id, EpisodeNumber::regular(1));
        episode.duracao_esperada = Some(1440);
        episode.progresso_atual = 1500; // Exceeds duration
        assert!(validate_episode(&episode).is_err());
    }
}