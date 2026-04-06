//! Task management types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Task status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
pub enum TaskStatus {
    /// Task created, not started
    Pending,
    /// Task in progress
    Processing,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
}

/// A background task for tracking async operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_id: String,
    pub task_type: String,
    pub status: TaskStatus,
    pub progress: u8,
    pub message: String,
    pub metadata: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Detailed progress info for frontend display
    pub progress_detail: Option<serde_json::Value>,
}

impl Task {
    /// Create a new task
    pub fn new(task_type: &str, metadata: serde_json::Value) -> Self {
        let now = Utc::now();
        Self {
            task_id: format!("task_{}", uuid::Uuid::new_v4().simple()),
            task_type: task_type.to_string(),
            status: TaskStatus::Pending,
            progress: 0,
            message: String::new(),
            metadata,
            result: None,
            error: None,
            created_at: now,
            updated_at: now,
            progress_detail: None,
        }
    }

    /// Convert to dictionary-style response for API
    pub fn to_dict(&self) -> TaskDictResponse {
        TaskDictResponse {
            task_id: self.task_id.clone(),
            task_type: self.task_type.clone(),
            status: self.status.to_string(),
            progress: self.progress,
            message: self.message.clone(),
            progress_detail: self.progress_detail.clone(),
            result: self.result.clone(),
            error: self.error.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// Task response as a dictionary (for API compatibility with Python version)
#[derive(Debug, Serialize)]
pub struct TaskDictResponse {
    pub task_id: String,
    pub task_type: String,
    pub status: String,
    pub progress: u8,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_detail: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Task creation request
#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub task_type: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Task status response
#[derive(Debug, Serialize)]
pub struct TaskStatusResponse {
    pub success: bool,
    pub data: TaskDictResponse,
}

/// Task list response
#[derive(Debug, Serialize)]
pub struct TaskListResponse {
    pub success: bool,
    pub data: Vec<TaskDictResponse>,
    pub count: usize,
}

/// Task progress update for SSE streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgressUpdate {
    pub task_id: String,
    pub progress: u8,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_detail: Option<serde_json::Value>,
}