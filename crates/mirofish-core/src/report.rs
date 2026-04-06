//! Report types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Report status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
pub enum ReportStatus {
    /// Report generation queued
    Pending,
    /// Planning report outline
    Planning,
    /// Generating sections
    Generating,
    /// Report completed
    Completed,
    /// Report generation failed
    Failed,
}

/// A section in the report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub index: usize,
    pub title: String,
    pub description: String,
    pub content: String,
    pub tool_calls_count: usize,
}

/// Report outline (generated during planning phase)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportOutline {
    pub title: String,
    pub summary: String,
    pub sections: Vec<ReportSectionOutline>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSectionOutline {
    pub title: String,
    pub description: String,
}

/// Full report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub report_id: String,
    pub simulation_id: String,
    pub graph_id: String,
    pub simulation_requirement: String,
    pub status: ReportStatus,
    pub outline: Option<ReportOutline>,
    pub sections: Vec<ReportSection>,
    pub markdown_content: String,
    pub progress_percent: u8,
    pub current_section: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Report {
    pub fn new(simulation_id: &str, graph_id: &str, simulation_requirement: &str) -> Self {
        let now = Utc::now();
        Self {
            report_id: format!("report_{}", uuid::Uuid::new_v4().simple()),
            simulation_id: simulation_id.to_string(),
            graph_id: graph_id.to_string(),
            simulation_requirement: simulation_requirement.to_string(),
            status: ReportStatus::Pending,
            outline: None,
            sections: Vec::new(),
            markdown_content: String::new(),
            progress_percent: 0,
            current_section: None,
            error: None,
            created_at: now,
            completed_at: None,
        }
    }
}

/// Chat message with report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// Chat request
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub history: Vec<ChatMessage>,
}

/// Chat response
#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub message: String,
    pub tools_used: Vec<String>,
}