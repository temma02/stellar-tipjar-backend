use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::db::connection::AppState;
use crate::upload::{FileValidator, ImageProcessor, S3Storage, UploadError};

#[derive(Serialize)]
pub struct UploadResponse {
    pub url: String,
    pub variants: Option<UploadVariants>,
}

#[derive(Serialize)]
pub struct UploadVariants {
    pub original: String,
    pub large: String,
    pub medium: String,
    pub thumbnail: String,
}

/// Upload creator profile image
pub async fn upload_profile_image(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, UploadError> {
    let validator = FileValidator::new().with_max_size(5 * 1024 * 1024); // 5MB
    
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        UploadError::ProcessingError(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "image" {
            let content_type = field.content_type()
                .ok_or_else(|| UploadError::InvalidFileType("Missing content type".to_string()))?
                .to_string();
            
            let data = field.bytes().await.map_err(|e| {
                UploadError::ProcessingError(format!("Failed to read file data: {}", e))
            })?;
            
            validator.validate(data.len(), &content_type)?;
            
            let variants = ImageProcessor::create_variants(&data)?;
            let storage = S3Storage::new().await?;
            
            let uploaded = storage.upload_variants(
                variants.original,
                variants.large,
                variants.medium,
                variants.thumbnail,
                &content_type,
                "profiles",
            ).await?;
            
            return Ok(Json(UploadResponse {
                url: uploaded.medium.clone(),
                variants: Some(UploadVariants {
                    original: uploaded.original,
                    large: uploaded.large,
                    medium: uploaded.medium,
                    thumbnail: uploaded.thumbnail,
                }),
            }));
        }
    }
    
    Err(UploadError::ProcessingError("No image field found".to_string()))
}

/// Upload creator banner image
pub async fn upload_banner_image(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, UploadError> {
    let validator = FileValidator::new().with_max_size(10 * 1024 * 1024); // 10MB for banners
    
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        UploadError::ProcessingError(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "banner" {
            let content_type = field.content_type()
                .ok_or_else(|| UploadError::InvalidFileType("Missing content type".to_string()))?
                .to_string();
            
            let data = field.bytes().await.map_err(|e| {
                UploadError::ProcessingError(format!("Failed to read file data: {}", e))
            })?;
            
            validator.validate(data.len(), &content_type)?;
            
            let processed = ImageProcessor::process(&data, 1920, 400)?;
            let storage = S3Storage::new().await?;
            
            let url = storage.upload(processed, &content_type, "banners").await?;
            
            return Ok(Json(UploadResponse {
                url,
                variants: None,
            }));
        }
    }
    
    Err(UploadError::ProcessingError("No banner field found".to_string()))
}
