use super::entity::Subtitle;
use crate::domain::{DomainError, DomainResult};

/// Validates all Subtitle invariants
pub fn validate_subtitle(subtitle: &Subtitle) -> DomainResult<()> {
    validate_language(subtitle)?;
    validate_version(subtitle)?;
    Ok(())
}

/// Language code cannot be empty
fn validate_language(subtitle: &Subtitle) -> DomainResult<()> {
    if subtitle.idioma.trim().is_empty() {
        return Err(DomainError::InvariantViolation(
            "Subtitle language cannot be empty".to_string()
        ));
    }
    Ok(())
}

/// Version must be positive
fn validate_version(subtitle: &Subtitle) -> DomainResult<()> {
    if subtitle.versao == 0 {
        return Err(DomainError::InvariantViolation(
            "Subtitle version must be at least 1".to_string()
        ));
    }
    Ok(())
}

/// Critical Subtitle Invariants:
/// 
/// 1. Every Subtitle MUST have a source File
/// 2. Original subtitle is NEVER overwritten
/// 3. Transformations ALWAYS create new versions
/// 4. Transformations are reversible (history preserved)
/// 5. Original files remain untouched on disk
/// 6. file_id is immutable (subtitle tied to specific file)
/// 7. Versions increment monotonically
/// 8. Original subtitles have eh_original = true, versao = 1

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::subtitle::{Subtitle, SubtitleFormat};
    use uuid::Uuid;

    #[test]
    fn test_valid_subtitle() {
        let file_id = Uuid::new_v4();
        let subtitle = Subtitle::new(
            file_id,
            SubtitleFormat::SRT,
            "pt-BR".to_string(),
        );
        assert!(validate_subtitle(&subtitle).is_ok());
        assert_eq!(subtitle.versao, 1);
        assert!(subtitle.eh_original);
    }

    #[test]
    fn test_derived_subtitle_increments_version() {
        let file_id = Uuid::new_v4();
        let original = Subtitle::new(
            file_id,
            SubtitleFormat::SRT,
            "en".to_string(),
        );
        
        let new_file_id = Uuid::new_v4();
        let derived = original.derive_from(new_file_id, SubtitleFormat::ASS);
        
        assert_eq!(derived.versao, 2);
        assert!(!derived.eh_original);
        assert_eq!(derived.idioma, original.idioma);
    }

    #[test]
    fn test_empty_language_fails() {
        let file_id = Uuid::new_v4();
        let subtitle = Subtitle::new(
            file_id,
            SubtitleFormat::SRT,
            "".to_string(),
        );
        assert!(validate_subtitle(&subtitle).is_err());
    }
}