//! MiroFish Text - Text processing and file parsing
//!
//! Provides:
//! - PDF text extraction
//! - Character encoding detection
//! - Text chunking for graph building
//! - File type detection and unified extraction

pub mod pdf;
pub mod encoding;
pub mod chunking;

pub use pdf::*;
pub use encoding::*;
pub use chunking::*;

/// Supported file types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Pdf,
    Text,
}

/// Extract text from file bytes based on file type
pub fn extract_text_from_file(data: &[u8], file_type: FileType) -> anyhow::Result<String> {
    match file_type {
        FileType::Pdf => {
            // Write to temp file and use pdf_extract
            let temp_path = std::env::temp_dir().join(format!("mirofish_{}.pdf", uuid::Uuid::new_v4()));
            std::fs::write(&temp_path, data)
                .map_err(|e| anyhow::anyhow!("Failed to write temp PDF: {}", e))?;
            
            let text = pdf_extract::extract_text(&temp_path)
                .map_err(|e| anyhow::anyhow!("Failed to extract PDF: {}", e))?;
            
            // Clean up temp file
            let _ = std::fs::remove_file(&temp_path);
            
            Ok(text.trim().to_string())
        }
        FileType::Text => {
            // Try UTF-8 first, then fall back to encoding detection
            if let Ok(s) = std::str::from_utf8(data) {
                Ok(s.to_string())
            } else {
                // Use encoding detection for non-UTF8 text
                let detected = detect_encoding(data);
                let text = String::from_utf8_lossy(data).to_string();
                Ok(format!("[Encoding: {}]\n{}", detected, text))
            }
        }
    }
}
