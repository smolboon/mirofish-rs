//! Project management types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Ontology;

/// Project status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
pub enum ProjectStatus {
    /// Project created, waiting for ontology generation
    Created,
    /// Ontology generated, ready for graph building
    OntologyGenerated,
    /// Graph building in progress
    GraphBuilding,
    /// Graph building completed
    GraphCompleted,
    /// Operation failed
    Failed,
}

/// File information for uploaded project files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub filename: String,
    pub size: u64,
    pub path: String,
    pub original_filename: String,
    pub uploaded_at: DateTime<Utc>,
}

/// A project represents a single simulation preparation workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub project_id: String,
    pub name: String,
    pub status: ProjectStatus,
    pub simulation_requirement: String,
    pub ontology: Option<Ontology>,
    pub analysis_summary: String,
    pub graph_id: Option<String>,
    pub graph_build_task_id: Option<String>,
    pub files: Vec<FileInfo>,
    pub total_text_length: usize,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    /// Create a new project
    pub fn new(name: &str) -> Self {
        let now = Utc::now();
        Self {
            project_id: format!("proj_{}", Uuid::new_v4().simple()),
            name: name.to_string(),
            status: ProjectStatus::Created,
            simulation_requirement: String::new(),
            ontology: None,
            analysis_summary: String::new(),
            graph_id: None,
            graph_build_task_id: None,
            files: Vec::new(),
            total_text_length: 0,
            chunk_size: 500,
            chunk_overlap: 50,
            error: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if project can accept graph building
    pub fn can_build_graph(&self) -> bool {
        matches!(
            self.status,
            ProjectStatus::OntologyGenerated | ProjectStatus::Failed
        )
    }

    /// Check if graph building is complete
    pub fn is_graph_complete(&self) -> bool {
        matches!(self.status, ProjectStatus::GraphCompleted)
    }
}

/// Project creation request
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub simulation_requirement: String,
    pub additional_context: Option<String>,
}

/// Project response
#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub success: bool,
    pub data: Project,
}

/// Project list response
#[derive(Debug, Serialize)]
pub struct ProjectListResponse {
    pub success: bool,
    pub data: Vec<Project>,
    pub count: usize,
}