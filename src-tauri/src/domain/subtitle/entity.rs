use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a subtitle as transformable data
/// Subtitles are versioned and never destructively edited
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtitle {
    /// Internal immutable identifier
    pub id: Uuid,
    
    /// Source file (REQUIRED)
    pub file_id: Uuid,
    
    /// Subtitle format
    pub formato: SubtitleFormat,
    
    /// Language (ISO 639-1 code preferred)
    pub idioma: String,
    
    /// Version identifier (for tracking transformations)
    pub versao: u32,
    
    /// Whether this is the original, unmodified subtitle
    pub eh_original: bool,
    
    /// Creation timestamp
    pub criado_em: DateTime<Utc>,
}

/// Supported subtitle formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SubtitleFormat {
    SRT,
    ASS,
    VTT,
}

/// Represents a transformation applied to a subtitle
/// Transformations create new subtitle versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleTransformation {
    /// Transformation identifier
    pub id: Uuid,
    
    /// Source subtitle that was transformed
    pub subtitle_id_origem: Uuid,
    
    /// Type of transformation
    pub tipo: TransformationType,
    
    /// Parameters applied (stored as JSON)
    pub parametros_aplicados: serde_json::Value,
    
    /// Creation timestamp
    pub criado_em: DateTime<Utc>,
}

/// Types of subtitle transformations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransformationType {
    /// Style changes (font, size, colors, outline)
    Style,
    
    /// Timing adjustments (sync, offset)
    Timing,
    
    /// Format conversion (SRT -> ASS, etc)
    Conversao,
}

impl Subtitle {
    /// Create a new Subtitle entity
    /// This is typically the original subtitle from a file
    pub fn new(
        file_id: Uuid,
        formato: SubtitleFormat,
        idioma: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            file_id,
            formato,
            idioma,
            versao: 1,
            eh_original: true,
            criado_em: Utc::now(),
        }
    }
    
    /// Create a derived subtitle from a transformation
    /// This increments the version and marks as non-original
    pub fn derive_from(&self, new_file_id: Uuid, formato: SubtitleFormat) -> Self {
        Self {
            id: Uuid::new_v4(),
            file_id: new_file_id,
            formato,
            idioma: self.idioma.clone(),
            versao: self.versao + 1,
            eh_original: false,
            criado_em: Utc::now(),
        }
    }
}

impl SubtitleTransformation {
    /// Create a new transformation record
    pub fn new(
        subtitle_id_origem: Uuid,
        tipo: TransformationType,
        parametros: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            subtitle_id_origem,
            tipo,
            parametros_aplicados: parametros,
            criado_em: Utc::now(),
        }
    }
}

impl SubtitleFormat {
    /// Infer format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "srt" => Some(SubtitleFormat::SRT),
            "ass" | "ssa" => Some(SubtitleFormat::ASS),
            "vtt" => Some(SubtitleFormat::VTT),
            _ => None,
        }
    }
}

impl std::fmt::Display for SubtitleFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubtitleFormat::SRT => write!(f, "SRT"),
            SubtitleFormat::ASS => write!(f, "ASS"),
            SubtitleFormat::VTT => write!(f, "VTT"),
        }
    }
}

impl std::fmt::Display for TransformationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransformationType::Style => write!(f, "style"),
            TransformationType::Timing => write!(f, "timing"),
            TransformationType::Conversao => write!(f, "conversao"),
        }
    }
}