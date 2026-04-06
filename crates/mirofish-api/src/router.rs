//! API router setup

use axum::{
    Router,
    routing::{get, post, delete},
};

use crate::state::AppState;
use crate::graph;
use crate::simulation;
use crate::report;
use crate::project;
use crate::upload;

/// Build the complete API router
pub fn build_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        // File upload endpoint
        .route("/api/upload", post(upload::upload_files))
        // Graph endpoints
        .route("/api/graph/ontology", post(graph::generate_ontology))
        .route("/api/graph/build", post(graph::build_graph))
        .route("/api/graph/task/{task_id}", get(graph::get_task_status))
        .route("/api/graph/task/{task_id}/stream", get(graph::stream_task))
        // Simulation endpoints
        .route("/api/simulation/create", post(simulation::create_simulation))
        .route("/api/simulation/prepare", post(simulation::prepare_simulation))
        .route("/api/simulation/start", post(simulation::start_simulation))
        .route("/api/simulation/stop", post(simulation::stop_simulation))
        .route("/api/simulation/status/{sim_id}", get(simulation::get_status))
        .route("/api/simulation/stream/{task_id}", get(simulation::stream_task))
        .route("/api/simulation/interview", post(simulation::interview_agent))
        .route("/api/simulation/interview/agents", get(simulation::list_agents))
        // Report endpoints
        .route("/api/report/generate", post(report::generate_report))
        .route("/api/report/status/{task_id}", get(report::get_status))
        .route("/api/report/stream/{task_id}", get(report::stream_task))
        .route("/api/report/chat", post(report::chat_with_report))
        .route("/api/report/list", get(report::list_reports))
        .route("/api/report/{report_id}", get(report::get_report))
        .route("/api/report/{report_id}", delete(report::delete_report))
        .route("/api/report/by-simulation/{simulation_id}", get(report::get_report_by_simulation))
        .route("/api/report/{report_id}/sections", get(report::get_report_sections))
        .route("/api/report/{report_id}/section/{section_index}", get(report::get_section))
        .route("/api/report/check/{simulation_id}", get(report::check_report_status))
        // Project endpoints
        .route("/api/project/create", post(project::create_project))
        .route("/api/project/list", get(project::list_projects))
        .route("/api/project/{project_id}", get(project::get_project))
        .route("/api/project/{project_id}/delete", post(project::delete_project))
        .with_state(state)
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}