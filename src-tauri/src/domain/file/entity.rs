use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Represents a physical file on disk
/// Files are observable entities, not controlled by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    /// Internal immutable identifier
    pub id: Uuid,

    /// Absolute path to the file
    pub caminho_absoluto: PathBuf,

    /// Type of file
    pub tipo: FileType,

    /// File size in bytes
    pub tamanho: u64,

    /// SHA256 hash (optional, computed on demand)
    pub hash: Option<String>,

    /// Last modification timestamp from filesystem
    pub data_modificacao: DateTime<Utc>,

    /// How this file was discovered
    pub origem: FileOrigin,

    /// Creation timestamp in our database
    pub criado_em: DateTime<Utc>,

    /// Last update timestamp in our database
    pub atualizado_em: DateTime<Utc>,
}

/// Type of file based on its purpose
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileType {
    Video,
    Legenda,
    Imagem,
    Outro,
}

/// How the file was discovered or added
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileOrigin {
    /// Discovered via directory scan
    Scan,

    /// Explicitly imported by user
    Importacao,

    /// Manually added
    Manual,
}

impl File {
    /// Create a new File entity
    pub fn new(
        caminho_absoluto: PathBuf,
        tipo: FileType,
        tamanho: u64,
        data_modificacao: DateTime<Utc>,
        origem: FileOrigin,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            caminho_absoluto,
            tipo,
            tamanho,
            hash: None,
            data_modificacao,
            origem,
            criado_em: now,
            atualizado_em: now,
        }
    }

    /// Update file metadata (size, modification date)
    /// This is called when the file changes on disk
    pub fn update_metadata(&mut self, tamanho: u64, data_modificacao: DateTime<Utc>) {
        self.tamanho = tamanho;
        self.data_modificacao = data_modificacao;
        // Hash is invalidated when file changes
        self.hash = None;
        self.atualizado_em = Utc::now();
    }

    /// Set the hash after computation
    pub fn set_hash(&mut self, hash: String) {
        self.hash = Some(hash);
        self.atualizado_em = Utc::now();
    }

    /// Check if file likely changed based on metadata
    pub fn has_changed(&self, tamanho: u64, data_modificacao: DateTime<Utc>) -> bool {
        self.tamanho != tamanho || self.data_modificacao != data_modificacao
    }
}

impl FileType {
    /// Infer file type from extension
    pub fn from_extension(path: &PathBuf) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("mkv") | Some("mp4") | Some("avi") | Some("webm") => FileType::Video,
            Some("srt") | Some("ass") | Some("vtt") => FileType::Legenda,
            Some("jpg") | Some("jpeg") | Some("png") | Some("webp") => FileType::Imagem,
            _ => FileType::Outro,
        }
    }
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Video => write!(f, "video"),
            FileType::Legenda => write!(f, "legenda"),
            FileType::Imagem => write!(f, "imagem"),
            FileType::Outro => write!(f, "outro"),
        }
    }
}

impl std::fmt::Display for FileOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileOrigin::Scan => write!(f, "scan"),
            FileOrigin::Importacao => write!(f, "importacao"),
            FileOrigin::Manual => write!(f, "manual"),
        }
    }
}
