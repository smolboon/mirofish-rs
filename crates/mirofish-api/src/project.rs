//! Project API handlers

use axum::{
    extract::{Path, State},
    Json,
};
use tracing::info;

use mirofish_core::{Project, CreateProjectRequest, ProjectResponse, ProjectListResponse};

use crate::state::AppState;

/// Create a new project
pub async fn create_project(
    State(_state): State<AppState>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<ProjectResponse>, (axum::http::StatusCode, String)> {
    info!("Creating project: {}", req.name);

    let mut project = Project::new(&req.name);
    project.simulation_requirement = req.simulation_requirement.clone();

    Ok(Json(ProjectResponse {
        success: true,
        data: project,
    }))
}

/// List all projects
pub async fn list_projects(
    State(_state): State<AppState>,
) -> Result<Json<ProjectListResponse>, (axum::http::StatusCode, String)> {
    Ok(Json(ProjectListResponse {
        success: true,
        data: Vec::new(),
        count: 0,
    }))
}

/// Get a project by ID
pub async fn get_project(
    State(_state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<ProjectResponse>, (axum::http::StatusCode, String)> {
    Err((
        axum::http::StatusCode::NOT_FOUND,
        format!("Project {} not found", project_id),
    ))
}

/// Delete a project
pub async fn delete_project(
    State(_state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    info!("Deleting project: {}", project_id);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Project {} deleted", project_id),
    })))
}