//! PDF text extraction

use std::path::Path;

use mirofish_core::FileError;

/// Extract text from a PDF file
pub fn extract_pdf(path: &Path) -> anyhow::Result<String> {
    if !path.exists() {
        return Err(FileError::NotFound(path.display().to_string()).into());
    }

    let text = pdf_extract::extract_text(path)
        .map_err(|e| FileError::ReadError(format!("Failed to extract PDF: {}", e)))?;

    Ok(text.trim().to_string())
}