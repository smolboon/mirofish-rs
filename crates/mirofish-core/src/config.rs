//! Application configuration

use serde::{Deserialize, Serialize};

/// Application configuration, loaded from environment or config files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Server host (default: 0.0.0.0)
    pub server_host: String,

    /// Server port (default: 5001)
    pub server_port: u16,

    /// Upload directory for files, projects, simulations, reports
    pub upload_folder: String,

    /// LLM API key
    pub llm_api_key: String,

    /// LLM base URL (OpenAI compatible)
    pub llm_base_url: String,

    /// LLM model name
    pub llm_model_name: String,

    /// Zep API key
    pub zep_api_key: String,

    /// Zep API base URL
    pub zep_base_url: String,

    /// Default chunk size for text processing
    pub default_chunk_size: usize,

    /// Default chunk overlap for text processing
    pub default_chunk_overlap: usize,

    /// Maximum upload file size (bytes)
    pub max_upload_size: usize,

    /// Allowed file extensions for upload
    pub allowed_extensions: Vec<String>,

    /// Default language for i18n
    pub default_language: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_host: "0.0.0.0".to_string(),
            server_port: 5001,
            upload_folder: "uploads".to_string(),
            llm_api_key: String::new(),
            llm_base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            llm_model_name: "qwen-plus".to_string(),
            zep_api_key: String::new(),
            zep_base_url: "https://api.getzep.com".to_string(),
            default_chunk_size: 500,
            default_chunk_overlap: 50,
            max_upload_size: 50 * 1024 * 1024, // 50MB
            allowed_extensions: vec![
                "pdf".to_string(),
                "md".to_string(),
                "txt".to_string(),
            ],
            default_language: "en".to_string(),
        }
    }
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, crate::ConfigError> {
        use std::env;

        let llm_api_key = env::var("LLM_API_KEY")
            .map_err(|_| crate::ConfigError::Missing("LLM_API_KEY".to_string()))?;

        let zep_api_key = env::var("ZEP_API_KEY")
            .map_err(|_| crate::ConfigError::Missing("ZEP_API_KEY".to_string()))?;

        Ok(Self {
            llm_api_key,
            llm_base_url: env::var("LLM_BASE_URL")
                .unwrap_or_else(|_| Self::default().llm_base_url),
            llm_model_name: env::var("LLM_MODEL_NAME")
                .unwrap_or_else(|_| Self::default().llm_model_name),
            zep_api_key,
            zep_base_url: env::var("ZEP_BASE_URL")
                .unwrap_or_else(|_| Self::default().zep_base_url),
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| Self::default().server_host),
            server_port: env::var("SERVER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(Self::default().server_port),
            upload_folder: env::var("UPLOAD_FOLDER")
                .unwrap_or_else(|_| Self::default().upload_folder),
            default_chunk_size: env::var("DEFAULT_CHUNK_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(Self::default().default_chunk_size),
            default_chunk_overlap: env::var("DEFAULT_CHUNK_OVERLAP")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(Self::default().default_chunk_overlap),
            max_upload_size: env::var("MAX_UPLOAD_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(Self::default().max_upload_size),
            allowed_extensions: env::var("ALLOWED_EXTENSIONS")
                .ok()
                .map(|s| s.split(',').map(String::from).collect())
                .unwrap_or_else(|| Self::default().allowed_extensions),
            default_language: env::var("DEFAULT_LANGUAGE")
                .unwrap_or_else(|_| Self::default().default_language),
        })
    }
}