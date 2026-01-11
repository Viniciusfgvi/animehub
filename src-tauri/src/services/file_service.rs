// src-tauri/src/services/file_service.rs
use crate::domain::file::{validate_file, File, FileOrigin, FileType};
use crate::error::{AppError, AppResult};
use crate::events::{DirectoryScanned, EventBus, FileDetected};
use crate::repositories::FileRepository;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

/// Request to register a detected file
#[derive(Debug, Clone)]
pub struct RegisterFileRequest {
    pub caminho_absoluto: PathBuf,
    pub tipo: FileType,
    pub tamanho: u64,
    pub data_modificacao: DateTime<Utc>,
    pub origem: FileOrigin,
    pub hash: Option<String>,
}

pub struct FileService {
    file_repo: Arc<dyn FileRepository>,
    event_bus: Arc<EventBus>,
}

impl FileService {
    pub fn new(file_repo: Arc<dyn FileRepository>, event_bus: Arc<EventBus>) -> Self {
        Self {
            file_repo,
            event_bus,
        }
    }

    pub fn register_file(&self, request: RegisterFileRequest) -> AppResult<Uuid> {
        let path_str = request.caminho_absoluto.to_string_lossy();
        if let Some(existing) = self.file_repo.get_by_path(&path_str)? {
            if existing.has_changed(request.tamanho, request.data_modificacao) {
                return self.update_file_metadata(
                    existing.id,
                    request.tamanho,
                    request.data_modificacao,
                );
            } else {
                return Ok(existing.id);
            }
        }

        let mut file = File::new(
            request.caminho_absoluto.clone(),
            request.tipo,
            request.tamanho,
            request.data_modificacao,
            request.origem,
        );

        if let Some(hash) = request.hash {
            file.set_hash(hash);
        }

        validate_file(&file).map_err(AppError::Domain)?;

        self.file_repo.save(&file)?;

        self.event_bus.emit(FileDetected::new(
            request.caminho_absoluto,
            request.tamanho,
            request.tipo.to_string(),
        ));

        Ok(file.id)
    }

    pub fn update_file_metadata(
        &self,
        file_id: Uuid,
        tamanho: u64,
        data_modificacao: DateTime<Utc>,
    ) -> AppResult<Uuid> {
        let mut file = self
            .file_repo
            .get_by_id(file_id)?
            .ok_or(AppError::NotFound)?;

        file.update_metadata(tamanho, data_modificacao);

        validate_file(&file).map_err(AppError::Domain)?;

        self.file_repo.save(&file)?;

        self.event_bus.emit(FileDetected::new(
            file.caminho_absoluto.clone(),
            file.tamanho,
            file.tipo.to_string(),
        ));

        Ok(file.id)
    }

    pub fn calculate_and_set_hash(&self, file_id: Uuid) -> AppResult<String> {
        let mut file = self
            .file_repo
            .get_by_id(file_id)?
            .ok_or(AppError::NotFound)?;

        let hash = self.calculate_file_hash(&file.caminho_absoluto)?;
        file.set_hash(hash.clone());
        self.file_repo.save(&file)?;

        Ok(hash)
    }

    pub fn get_file(&self, file_id: Uuid) -> AppResult<Option<File>> {
        self.file_repo.get_by_id(file_id)
    }

    pub fn scan_directory(&self, directory_path: PathBuf) -> AppResult<usize> {
        if !directory_path.exists() {
            return Err(AppError::Other("Directory does not exist".to_string()));
        }
        if !directory_path.is_dir() {
            return Err(AppError::Other("Path is not a directory".to_string()));
        }

        let mut files_found = 0;

        for entry in walkdir::WalkDir::new(&directory_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e: Result<walkdir::DirEntry, walkdir::Error>| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path().to_path_buf();
                let file_type = FileType::from_extension(&path);

                if matches!(
                    file_type,
                    FileType::Video | FileType::Legenda | FileType::Imagem
                ) {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        self.event_bus.emit(FileDetected::new(
                            path.clone(),
                            metadata.len(),
                            file_type.to_string(),
                        ));
                        let request = RegisterFileRequest {
                            caminho_absoluto: path,
                            tipo: file_type,
                            tamanho: metadata.len(),
                            data_modificacao: chrono::DateTime::from(
                                metadata.modified().unwrap_or(std::time::SystemTime::now()),
                            ),
                            origem: FileOrigin::Scan,
                            hash: None,
                        };
                        let _ = self.register_file(request);
                        files_found += 1;
                    }
                }
            }
        }

        self.event_bus
            .emit(DirectoryScanned::new(directory_path, files_found));
        Ok(files_found)
    }

    fn calculate_file_hash(&self, path: &std::path::PathBuf) -> AppResult<String> {
        use sha2::{Digest, Sha256};
        use std::fs::File as StdFile;
        use std::io::Read;

        let mut file = StdFile::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }
}
