use super::entity::Anime;
use crate::domain::{DomainError, DomainResult};

/// Validates all Anime invariants
/// These are the absolute rules that must hold for an Anime to be valid
pub fn validate_anime(anime: &Anime) -> DomainResult<()> {
    validate_titulo_principal(&anime.titulo_principal)?;
    validate_dates(anime)?;
    Ok(())
}

/// Titulo principal cannot be empty
fn validate_titulo_principal(titulo: &str) -> DomainResult<()> {
    if titulo.trim().is_empty() {
        return Err(DomainError::InvariantViolation(
            "Anime title cannot be empty".to_string(),
        ));
    }
    Ok(())
}

/// If both dates are present, start must be before or equal to end
fn validate_dates(anime: &Anime) -> DomainResult<()> {
    if let (Some(inicio), Some(fim)) = (anime.data_inicio, anime.data_fim) {
        if inicio > fim {
            return Err(DomainError::InvariantViolation(format!(
                "Start date {:?} cannot be after end date {:?}",
                inicio, fim
            )));
        }
    }
    Ok(())
}

/// Invariants that must hold true for Anime domain:
///
/// 1. Anime can exist without episodes
/// 2. Anime can exist without files
/// 3. Anime can exist without external references
/// 4. Identity (UUID) is immutable
/// 5. Duplicates are allowed until explicit resolution
/// 6. Title cannot be empty
/// 7. If both dates exist, start <= end
/// 8. Created timestamp never changes
/// 9. Updated timestamp reflects last modification

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::anime::{AnimeStatus, AnimeType};

    #[test]
    fn test_valid_anime() {
        let anime = Anime::new("Steins;Gate".to_string(), AnimeType::TV);
        assert!(validate_anime(&anime).is_ok());
    }

    #[test]
    fn test_empty_title_fails() {
        let anime = Anime::new("   ".to_string(), AnimeType::TV);
        assert!(validate_anime(&anime).is_err());
    }
}
