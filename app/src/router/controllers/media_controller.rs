use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
};
use image;
use std::path::PathBuf;
use tokio::fs;

use crate::app::App;
use crate::database::ImageJobRepository;
use appkit_core::error::AppError;
use appkit_core::image::{ImageProcessor, ThumbOptions};

#[axum::debug_handler]
pub async fn generate_thumb(
    State(state): State<App>,
    Path((disk, hash, filename)): Path<(String, String, String)>,
) -> Result<Response, AppError> {
    // Construct media root directory
    let media_root = PathBuf::from("public")
        .join("media")
        .join("images")
        .join(&disk)
        .join(&hash);

    // Try to load job options
    let options = match load_job_options(&state, &disk, &filename, &media_root).await {
        Ok(opts) => opts,
        Err(e) => {
            tracing::error!("Failed to load job options for {}: {}", filename, e);
            return Err(AppError::NotFound(format!(
                "Thumbnail job not found: {}",
                filename
            )));
        }
    };

    // Get original filename from options
    let original_filename = options
        .get("filename")
        .and_then(|f| f.as_str())
        .unwrap_or(&filename);

    // Construct original file path
    let original = PathBuf::from("public/uploads").join(original_filename);

    if !original.exists() {
        return Err(AppError::NotFound(format!(
            "Original file not found: {}",
            original_filename
        )));
    }

    // Destination path for thumbnail
    let dst = media_root.join(&filename);

    // Generate thumbnail if it doesn't exist
    if !dst.exists() {
        tracing::info!("Generating thumbnail: {}", filename);

        // Create directory if needed
        if let Err(e) = fs::create_dir_all(&media_root).await {
            return Err(AppError::InternalError(format!(
                "Failed to create media directory: {}",
                e
            )));
        }

        // Copy original and process
        if let Err(e) = fs::copy(&original, &dst).await {
            return Err(AppError::InternalError(format!(
                "Failed to copy original file: {}",
                e
            )));
        }

        // Process thumbnail
        if let Err(e) = process_thumbnail(&dst, &options).await {
            let error_msg = e.to_string();
            tracing::error!("Failed to process thumbnail {}: {}", filename, error_msg);
            // Clean up failed thumbnail
            let _ = fs::remove_file(&dst).await;

            // Mark job as failed in database
            let job_repo = ImageJobRepository::new(state.db_pool.clone());
            if let Ok(Some(job)) = job_repo.find_by_filename(&filename, &disk).await {
                let _ = job_repo.mark_failed(job.id, &error_msg).await;
            }

            return Err(AppError::InternalError(format!(
                "Failed to process thumbnail: {}",
                error_msg
            )));
        }

        // Mark job as processed in database
        let job_repo = ImageJobRepository::new(state.db_pool.clone());
        if let Ok(Some(job)) = job_repo.find_by_filename(&filename, &disk).await {
            let _ = job_repo.mark_processed(job.id).await;
        }

        tracing::info!("Thumbnail generated successfully: {}", filename);
    } else {
        // Mark job as accessed (for analytics/cleanup)
        let job_repo = ImageJobRepository::new(state.db_pool.clone());
        if let Ok(Some(job)) = job_repo.find_by_filename(&filename, &disk).await {
            let _ = job_repo.mark_accessed(job.id).await;
        }
    }

    // Serve the file
    serve_image_file(&dst).await
}

/// Load job options from database or JSON file (fallback)
async fn load_job_options(
    state: &App,
    disk: &str,
    filename: &str,
    media_root: &PathBuf,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Try database first
    let job_repo = ImageJobRepository::new(state.db_pool.clone());
    if let Ok(Some(job)) = job_repo.find_by_filename(filename, disk).await {
        return job.get_options().map_err(|e| e.into());
    }

    // Fall back to JSON file
    let job_file = media_root.join(".jobs").join(format!("{}.json", filename));

    if !job_file.exists() {
        return Err(format!("Job file not found: {:?}", job_file).into());
    }

    let content = fs::read_to_string(&job_file).await?;
    let options: serde_json::Value = serde_json::from_str(&content)?;

    Ok(options)
}

/// Process thumbnail with options
async fn process_thumbnail(
    path: &PathBuf,
    options: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let width = options
        .get("width")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    let height = options
        .get("height")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    let quality = options
        .get("quality")
        .and_then(|v| v.as_u64())
        .map(|v| v as u8);
    let crop = options.get("crop").and_then(|v| v.as_str());
    let blur = options
        .get("blur")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    let grayscale = options
        .get("grayscale")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Build ThumbOptions
    let mut thumb_options = ThumbOptions::builder();

    if let Some(w) = width {
        thumb_options = thumb_options.width(w);
    }
    if let Some(h) = height {
        thumb_options = thumb_options.height(h);
    }
    if let Some(q) = quality {
        thumb_options = thumb_options.quality(q);
    }
    if crop.is_some() {
        thumb_options = thumb_options.crop(appkit_core::image::CropPosition::Center);
    }
    if let Some(b) = blur {
        thumb_options = thumb_options.blur(b);
    }
    if grayscale {
        thumb_options = thumb_options.grayscale();
    }

    let options = thumb_options.build();

    // Process in blocking task
    let path = path.clone();
    tokio::task::spawn_blocking(move || {
        let img = image::open(&path)?;
        let processed = ImageProcessor::process_image(img, options)?;
        processed.save(&path)?;
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    })
    .await
    .map_err(|e| AppError::InternalError(format!("Task join error: {}", e)))?
    .map_err(|e| AppError::InternalError(format!("Image processing error: {}", e)))?;

    Ok(())
}

/// Serve an image file with CDN-friendly headers
async fn serve_image_file(path: &PathBuf) -> Result<Response, AppError> {
    let content = fs::read(path)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read file: {}", e)))?;

    // Detect MIME type
    let mime_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .header(header::CACHE_CONTROL, "public, max-age=31536000") // 1 year
        .header(header::ETAG, format!("{:x}", md5::compute(&content))) // ETag for caching
        .body(Body::from(content))
        .unwrap())
}
