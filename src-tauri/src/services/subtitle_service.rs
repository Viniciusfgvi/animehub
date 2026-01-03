// src-tauri/src/services/subtitle_service.rs
use std::sync::Arc;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use crate::domain::subtitle::{SubtitleFormat, SubtitleTransformation, TransformationType};
use crate::repositories::{SubtitleRepository, FileRepository};
use crate::events::{EventBus, SubtitleStyleApplied, SubtitleTimingAdjusted, SubtitleVersionCreated};
use crate::infrastructure::subtitle_workspace::{SubtitleWorkspace, SubtitleWorkspaceCreated, SubtitleWorkspaceCleaned};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct StyleTransformRequest {
    pub subtitle_id: Uuid,
    pub font_name: Option<String>,
    pub font_size: Option<u32>,
    pub primary_color: Option<String>,
    pub outline_color: Option<String>,
    pub outline_width: Option<f32>,
    pub shadow_offset: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct TimingTransformRequest {
    pub subtitle_id: Uuid,
    pub offset_ms: i64,
}

pub struct SubtitleService {
    subtitle_repo: Arc<dyn SubtitleRepository>,
    file_repo: Arc<dyn FileRepository>,
    event_bus: Arc<EventBus>,
}

impl SubtitleService {
    pub fn new(
        subtitle_repo: Arc<dyn SubtitleRepository>,
        file_repo: Arc<dyn FileRepository>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            subtitle_repo,
            file_repo,
            event_bus,
        }
    }

    pub fn apply_style_transformation(&self, request: StyleTransformRequest) -> AppResult<Uuid> {
        let params = serde_json::json!({
            "font_name": request.font_name,
            "font_size": request.font_size,
        });

        self.execute_transformation_pipeline(
            request.subtitle_id,
            TransformationType::Style,
            params,
            |path, format| self.apply_style_to_file(path, format, &request),
            |original_id, new_id| {
                self.event_bus.emit(SubtitleStyleApplied::new(original_id, new_id));
            }
        )
    }

    pub fn apply_timing_transformation(&self, request: TimingTransformRequest) -> AppResult<Uuid> {
        let params = serde_json::json!({ "offset_ms": request.offset_ms });
        self.execute_transformation_pipeline(
            request.subtitle_id,
            TransformationType::Timing,
            params,
            |path, format| self.apply_timing_to_file(path, format, request.offset_ms),
            |original_id, new_id| {
                self.event_bus.emit(SubtitleTimingAdjusted::new(original_id, new_id, request.offset_ms));
            }
        )
    }

    pub fn get_transformation_history(&self, subtitle_id: Uuid) -> AppResult<Vec<SubtitleTransformation>> {
        self.subtitle_repo.get_transformations(subtitle_id)
    }

    fn execute_transformation_pipeline<F, E>(
        &self,
        subtitle_id: Uuid,
        trans_type: TransformationType,
        params: serde_json::Value,
        transform_fn: F,
        emit_custom_event: E,
    ) -> AppResult<Uuid> 
    where 
        F: FnOnce(&Path, &SubtitleFormat) -> AppResult<()>,
        E: FnOnce(Uuid, Uuid),
    {
        let original = self.subtitle_repo
            .get_subtitle_by_id(subtitle_id)?
            .ok_or(AppError::NotFound)?;

        let original_file = self.file_repo
            .get_by_id(original.file_id)?
            .ok_or(AppError::NotFound)?;

        let mut workspace = SubtitleWorkspace::new(original_file.caminho_absoluto.clone())?;
        self.event_bus.emit(SubtitleWorkspaceCreated::new(workspace.id, original.id));

        transform_fn(workspace.working_file_path(), &original.formato)?;

        let new_file_path = self.generate_versioned_path(&original_file.caminho_absoluto, original.versao + 1);
        workspace.copy_working_file_to(&new_file_path)?;

        let file_metadata = std::fs::metadata(&new_file_path)?;
        let new_file = crate::domain::file::File::new(
            new_file_path,
            crate::domain::file::FileType::Legenda,
            file_metadata.len(),
            chrono::DateTime::from(file_metadata.modified().unwrap_or(std::time::SystemTime::now())),
            crate::domain::file::FileOrigin::Manual,
        );

        self.file_repo.save(&new_file)?;

        let new_subtitle = original.derive_from(new_file.id, original.formato);
        self.subtitle_repo.save_subtitle(&new_subtitle)?;

        let transformation = SubtitleTransformation::new(original.id, trans_type, params);
        self.subtitle_repo.save_transformation(&transformation)?;

        emit_custom_event(original.id, new_subtitle.id);
        self.event_bus.emit(SubtitleVersionCreated::new(new_subtitle.id, new_subtitle.versao));

        workspace.cleanup()?;
        self.event_bus.emit(SubtitleWorkspaceCleaned::new(workspace.id));

        Ok(new_subtitle.id)
    }

    fn apply_style_to_file(&self, file_path: &Path, format: &SubtitleFormat, request: &StyleTransformRequest) -> AppResult<()> {
        match format {
            SubtitleFormat::ASS => self.apply_style_to_ass(file_path, request),
            _ => Ok(()),
        }
    }

    fn apply_timing_to_file(&self, file_path: &Path, format: &SubtitleFormat, offset_ms: i64) -> AppResult<()> {
        match format {
            SubtitleFormat::ASS => self.apply_timing_to_ass(file_path, offset_ms),
            SubtitleFormat::SRT | SubtitleFormat::VTT => self.apply_timing_to_srt(file_path, offset_ms),
        }
    }

    fn generate_versioned_path(&self, original: &PathBuf, version: u32) -> PathBuf {
        let stem = original.file_stem().and_then(|s| s.to_str()).unwrap_or("subtitle");
        let ext = original.extension().and_then(|e| e.to_str()).unwrap_or("srt");
        let parent = original.parent().unwrap_or(Path::new("."));
        parent.join(format!("{}.v{}.{}", stem, version, ext))
    }

    fn apply_style_to_ass(&self, path: &Path, request: &StyleTransformRequest) -> AppResult<()> {
        let content = std::fs::read_to_string(path)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut in_styles = false;

        for line in lines.iter_mut() {
            if line.starts_with("[V4+ Styles]") {
                in_styles = true;
                continue;
            }
            if in_styles && line.starts_with("Style:") {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() > 10 {
                    let mut new_parts: Vec<String> = parts.iter().map(|&s| s.to_string()).collect();
                    if let Some(ref font) = request.font_name {
                        new_parts[1] = font.clone();
                    }
                    if let Some(size) = request.font_size {
                        new_parts[2] = size.to_string();
                    }
                    *line = new_parts.join(",");
                }
            }
            if in_styles && line.is_empty() {
                in_styles = false;
            }
        }
        std::fs::write(path, lines.join("\n"))?;
        Ok(())
    }

    fn apply_timing_to_ass(&self, _path: &Path, _offset_ms: i64) -> AppResult<()> { Ok(()) }
    fn apply_timing_to_srt(&self, _path: &Path, _offset_ms: i64) -> AppResult<()> { Ok(()) }
}