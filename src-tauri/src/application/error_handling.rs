// src-tauri/src/application/error_handling.rs
//
// Enhanced Error Handling for Commands
//
// ARCHITECTURE:
// - Maps internal errors → user-friendly responses
// - Provides consistent error format for UI
// - Never exposes internal implementation details
// - Logs errors for debugging

use serde::{Serialize, Deserialize};
use crate::error::AppError;

/// Standard error response for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error_type: ErrorType,
    pub message: String,
    pub details: Option<String>,
}

/// Error categories for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    /// Resource not found (404)
    NotFound,
    
    /// Invalid input/validation error (400)
    Validation,
    
    /// Domain invariant violation (422)
    DomainError,
    
    /// Database/persistence error (500)
    Database,
    
    /// External service error (502)
    ExternalService,
    
    /// File system error (500)
    FileSystem,
    
    /// Other/unknown error (500)
    Internal,
}

impl ErrorResponse {
    /// Create error response from AppError
    pub fn from_app_error(error: AppError) -> Self {
        match error {
            AppError::NotFound => Self {
                success: false,
                error_type: ErrorType::NotFound,
                message: "Resource not found".to_string(),
                details: None,
            },
            
            AppError::Domain(domain_error) => Self {
                success: false,
                error_type: ErrorType::DomainError,
                message: "Domain validation failed".to_string(),
                details: Some(domain_error.to_string()),
            },
            
            AppError::Database(db_error) => {
                // Log full error for debugging
                eprintln!("Database error: {:?}", db_error);
                
                Self {
                    success: false,
                    error_type: ErrorType::Database,
                    message: "Database operation failed".to_string(),
                    details: Some("Check logs for details".to_string()),
                }
            },
            
            AppError::Serialization(serde_error) => {
                eprintln!("Serialization error: {:?}", serde_error);
                
                Self {
                    success: false,
                    error_type: ErrorType::Internal,
                    message: "Data serialization failed".to_string(),
                    details: None,
                }
            },
            
            AppError::Io(io_error) => {
                eprintln!("IO error: {:?}", io_error);
                
                Self {
                    success: false,
                    error_type: ErrorType::FileSystem,
                    message: "File system operation failed".to_string(),
                    details: Some(io_error.to_string()),
                }
            },
            
            AppError::Pool(pool_error) => {
                eprintln!("Connection pool error: {}", pool_error);
                
                Self {
                    success: false,
                    error_type: ErrorType::Database,
                    message: "Database connection failed".to_string(),
                    details: None,
                }
            },
            
            AppError::Other(message) => {
                // Check if it's an external service error
                if message.contains("AniList") || message.contains("MPV") {
                    Self {
                        success: false,
                        error_type: ErrorType::ExternalService,
                        message: "External service error".to_string(),
                        details: Some(message),
                    }
                } else {
                    eprintln!("Other error: {}", message);
                    
                    Self {
                        success: false,
                        error_type: ErrorType::Internal,
                        message,
                        details: None,
                    }
                }
            },
        }
    }
    
    /// Create validation error
    pub fn validation(message: String) -> Self {
        Self {
            success: false,
            error_type: ErrorType::Validation,
            message,
            details: None,
        }
    }
    
    /// Create not found error
    pub fn not_found(resource: &str) -> Self {
        Self {
            success: false,
            error_type: ErrorType::NotFound,
            message: format!("{} not found", resource),
            details: None,
        }
    }
}

/// Helper trait to convert Results to ErrorResponse
pub trait ToErrorResponse<T> {
    fn to_error_response(self) -> Result<T, String>;
}

impl<T> ToErrorResponse<T> for Result<T, AppError> {
    fn to_error_response(self) -> Result<T, String> {
        self.map_err(|e| {
            let error_response = ErrorResponse::from_app_error(e);
            serde_json::to_string(&error_response)
                .unwrap_or_else(|_| "Internal error".to_string())
        })
    }
}

/// Macro to wrap command results with error handling
#[macro_export]
macro_rules! handle_command {
    ($expr:expr) => {
        match $expr {
            Ok(value) => Ok(value),
            Err(e) => {
                let error_response = ErrorResponse::from_app_error(e);
                Err(serde_json::to_string(&error_response)
                    .unwrap_or_else(|_| "Internal error".to_string()))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_not_found_error() {
        let error = ErrorResponse::from_app_error(AppError::NotFound);
        assert_eq!(error.error_type, ErrorType::NotFound);
        assert_eq!(error.message, "Resource not found");
    }
    
    #[test]
    fn test_validation_error() {
        let error = ErrorResponse::validation("Invalid input".to_string());
        assert_eq!(error.error_type, ErrorType::Validation);
        assert_eq!(error.message, "Invalid input");
    }
    
    #[test]
    fn test_serialization() {
        let error = ErrorResponse::not_found("Anime");
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("not_found"));
        assert!(json.contains("Anime not found"));
    }
}