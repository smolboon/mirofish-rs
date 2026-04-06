//! Graph API handlers

use axum::{
    extract::{Path, State},
    Json,
};
use tracing::info;

use mirofish_core::{OntologyRequest, OntologyResponse, GraphBuildRequest, GraphBuildResponse};
use mirofish_graph::builder::GraphBuilder;

use crate::state::AppState;

/// Generate ontology from text
pub async fn generate_ontology(
    State(state): State<AppState>,
    Json(req): Json<OntologyRequest>,
) -> Result<Json<OntologyResponse>, (axum::http::StatusCode, String)> {
    info!("Generating ontology");

    let ontology = mirofish_graph::ontology::generate_ontology(
        &state.llm,
        &req.analysis_summary,
        "",
    )
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(OntologyResponse {
        ontology,
        message: "Ontology generated successfully".to_string(),
    }))
}

/// Build graph from text
pub async fn build_graph(
    State(state): State<AppState>,
    Json(req): Json<GraphBuildRequest>,
) -> Result<Json<GraphBuildResponse>, (axum::http::StatusCode, String)> {
    info!("Building graph for project: {}", req.project_id);

    let task_id = state.task_manager.create_task(
        "graph_build",
        serde_json::json!({
            "project_id": req.project_id,
        }),
    );

    // Spawn background task
    let config = state.config.clone();
    let task_manager = state.task_manager.clone();
    let task_id_clone = task_id.clone();
    let text = req.document_text.clone();
    let project_id = req.project_id.clone();

    tokio::spawn(async move {
        let builder = GraphBuilder::new(&config);

        match builder.build_graph(&project_id, &mirofish_core::Ontology::empty(), vec![text], |msg, progress| {
            let _ = task_manager.update_task(
                &task_id_clone,
                None,
                Some((progress * 100.0) as u8),
                Some(msg),
                None,
            );
        }).await {
            Ok(_) => {
                task_manager.complete_task(&task_id_clone, serde_json::json!({ "status": "completed" }));
            }
            Err(e) => {
                task_manager.fail_task(&task_id_clone, &e.to_string());
            }
        }
    });

    Ok(Json(GraphBuildResponse {
        graph_id: format!("graph_{}", uuid::Uuid::new_v4().simple()),
        node_count: 0,
        edge_count: 0,
        entity_types: vec![],
        ontology: mirofish_core::Ontology::empty(),
    }))
}

/// Get task status
pub async fn get_task_status(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let task = state.task_manager.get_task(&task_id)
        .ok_or((axum::http::StatusCode::NOT_FOUND, "Task not found".to_string()))?;

    let dict = task.to_dict();
    Ok(Json(serde_json::json!({
        "task_id": dict.task_id,
        "task_type": dict.task_type,
        "status": dict.status,
        "progress": dict.progress,
        "message": dict.message,
        "metadata": task.metadata,
        "created_at": dict.created_at.to_rfc3339(),
    })))
}

/// Stream task progress via SSE
pub async fn stream_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let task = state.task_manager.get_task(&task_id);
    match task {
        Some(_) => {
            // For now, return a simple response - SSE streaming needs watch channel
            Ok(Json(serde_json::json!({
                "event": "stream_started",
                "task_id": task_id,
            })))
        }
        None => {
            Err((axum::http::StatusCode::NOT_FOUND, "Task not found".to_string()))
        }
    }
}
