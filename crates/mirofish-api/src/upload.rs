//! File upload handlers

use axum::{
    extract::{Multipart, State},
    Json,
};
use tracing::info;

use mirofish_text::{extract_text_from_file, FileType};

use crate::state::AppState;

/// Upload files and extract text
pub async fn upload_files(
    State(_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    info!("Handling file upload");

    let mut uploaded_files = Vec::new();
    let mut combined_text = String::new();

    while let Some(field) = multipart.next_field().await
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, format!("Failed to read multipart field: {}", e)))?
    {
        let filename = field.file_name()
            .map(String::from)
            .unwrap_or_else(|| "unknown".to_string());
        
        let content_type = field.content_type()
            .map(String::from);

        let data = field.bytes().await
            .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, format!("Failed to read file data: {}", e)))?;

        // Determine file type
        let file_type = match &content_type {
            Some(ct) if ct.contains("pdf") => FileType::Pdf,
            Some(ct) if ct.contains("text") || ct.contains("markdown") => FileType::Text,
            _ => {
                // Try to detect from extension
                if filename.ends_with(".pdf") {
                    FileType::Pdf
                } else if filename.ends_with(".md") || filename.ends_with(".txt") {
                    FileType::Text
                } else {
                    FileType::Text // Default to text
                }
            }
        };

        // Extract text from file
        let text = extract_text_from_file(&data, file_type)
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to extract text: {}", e)))?;

        combined_text.push_str(&text);
        combined_text.push_str("\n\n---\n\n");

        uploaded_files.push(serde_json::json!({
            "filename": filename,
            "size": data.len(),
            "text_length": text.len(),
        }));
    }

    if uploaded_files.is_empty() {
        return Err((axum::http::StatusCode::BAD_REQUEST, "No files uploaded".to_string()));
    }

    info!("Uploaded {} files, extracted {} characters", uploaded_files.len(), combined_text.len());

    Ok(Json(serde_json::json!({
        "files": uploaded_files,
        "combined_text_length": combined_text.len(),
        "message": format!("Successfully uploaded {} files", uploaded_files.len()),
    })))
}