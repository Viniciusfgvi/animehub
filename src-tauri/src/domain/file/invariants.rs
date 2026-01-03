use super::entity::File;
use crate::domain::{DomainError, DomainResult};

/// Validates all File invariants
pub fn validate_file(file: &File) -> DomainResult<()> {
    validate_path(file)?;
    Ok(())
}

/// Path must be absolute and non-empty
fn validate_path(file: &File) -> DomainResult<()> {
    if !file.caminho_absoluto.is_absolute() {
        return Err(DomainError::InvariantViolation(
            format!("File path must be absolute: {:?}", file.caminho_absoluto)
        ));
    }
    
    if file.caminho_absoluto.as_os_str().is_empty() {
        return Err(DomainError::InvariantViolation(
            "File path cannot be empty".to_string()
        ));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::file::{File, FileType, FileOrigin};
    use std::path::PathBuf;
    use chrono::Utc;

    #[test]
    fn test_valid_file() {
        // Solução multiplataforma: usa o diretório atual do sistema para garantir um path absoluto
        let mut path = std::env::current_dir().unwrap();
        path.push("episode1.mkv");

        let file = File::new(
            path,
            FileType::Video,
            1024 * 1024 * 500, // 500MB
            Utc::now(),
            FileOrigin::Scan,
        );
        assert!(validate_file(&file).is_ok());
    }

    #[test]
    fn test_relative_path_fails() {
        let file = File::new(
            PathBuf::from("relative/path/ep1.mkv"),
            FileType::Video,
            1024,
            Utc::now(),
            FileOrigin::Manual,
        );
        
        let result = validate_file(&file);
        assert!(result.is_err());
        
        if let Err(DomainError::InvariantViolation(msg)) = result {
            assert!(msg.contains("must be absolute"));
        } else {
            panic!("Expected InvariantViolation error");
        }
    }
}