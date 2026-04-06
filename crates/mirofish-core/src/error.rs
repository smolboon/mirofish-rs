//! Error types for MiroFish

use thiserror::Error;

/// Root error type for all MiroFish errors
#[derive(Error, Debug)]
pub enum MiroFishError {
    #[error("LLM error: {0}")]
    Llm(#[from] LlmError),

    #[error("Graph error: {0}")]
    Graph(#[from] GraphError),

    #[error("Simulation error: {0}")]
    Simulation(#[from] SimulationError),

    #[error("Report error: {0}")]
    Report(#[from] ReportError),

    #[error("Task error: {0}")]
    Task(#[from] TaskError),

    #[error("File error: {0}")]
    File(#[from] FileError),

    #[error("Config error: {0}")]
    Config(#[from] ConfigError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl MiroFishError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }
}

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("LLM API error: {0}")]
    Api(String),

    #[error("LLM response parsing error: {0}")]
    ParseError(String),

    #[error("LLM rate limit exceeded")]
    RateLimit,

    #[error("LLM timeout: {0}")]
    Timeout(String),
}

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Graph not found: {0}")]
    NotFound(String),

    #[error("Zep API error: {0}")]
    ZepApi(String),

    #[error("Ontology error: {0}")]
    Ontology(String),

    #[error("Entity not found: {0}")]
    EntityNotFound(String),
}

#[derive(Error, Debug)]
pub enum SimulationError {
    #[error("Simulation not found: {0}")]
    NotFound(String),

    #[error("Simulation already running: {0}")]
    AlreadyRunning(String),

    #[error("Simulation not ready: {0}")]
    NotReady(String),

    #[error("Agent error: {0}")]
    Agent(String),

    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Interview error: {0}")]
    Interview(String),

    #[error("Interview timeout")]
    InterviewTimeout,
}

#[derive(Error, Debug)]
pub enum ReportError {
    #[error("Report not found: {0}")]
    NotFound(String),

    #[error("Report generation failed: {0}")]
    GenerationFailed(String),

    #[error("Tool execution failed: {tool_name}: {error}")]
    ToolFailed { tool_name: String, error: String },

    #[error("Max iterations reached")]
    MaxIterationsReached,
}

#[derive(Error, Debug)]
pub enum TaskError {
    #[error("Task not found: {0}")]
    NotFound(String),

    #[error("Task already completed")]
    AlreadyCompleted,

    #[error("Task failed: {0}")]
    Failed(String),
}

#[derive(Error, Debug)]
pub enum FileError {
    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Unsupported file type: {0}")]
    UnsupportedType(String),

    #[error("File read error: {0}")]
    ReadError(String),

    #[error("Encoding detection failed: {0}")]
    EncodingError(String),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing config: {0}")]
    Missing(String),

    #[error("Invalid config: {0}")]
    Invalid(String),
}