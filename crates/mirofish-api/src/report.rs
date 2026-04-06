//! Report API handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use tracing::info;

use mirofish_core::{ChatRequest, ChatResponse};
use mirofish_report::ReportAgent;

use crate::state::AppState;

/// Generate a report
pub async fn generate_report(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let simulation_id = req.get("simulation_id")
        .and_then(|v| v.as_str())
        .ok_or((axum::http::StatusCode::BAD_REQUEST, "Missing simulation_id".to_string()))?
        .to_string();

    let graph_id = req.get("graph_id")
        .and_then(|v| v.as_str())
        .ok_or((axum::http::StatusCode::BAD_REQUEST, "Missing graph_id".to_string()))?
        .to_string();

    let simulation_requirement = req.get("simulation_requirement")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    info!("Generating report for simulation: {}", simulation_id);

    let task_id = state.task_manager.create_task(
        "report_generation",
        serde_json::json!({
            "simulation_id": simulation_id,
            "graph_id": graph_id,
        }),
    );

    let llm = state.llm.clone();
    let zep = state.zep.clone();
    let task_manager = state.task_manager.clone();
    let task_manager_for_complete = state.task_manager.clone();
    let task_id_clone = task_id.clone();

    tokio::spawn(async move {
        let report_agent = ReportAgent::new(
            llm,
            zep,
            task_manager,
        );

        match report_agent.generate_report(&simulation_id, &graph_id, &simulation_requirement, &task_id_clone).await {
            Ok(report) => {
                task_manager_for_complete.complete_task(
                    &task_id_clone,
                    serde_json::json!({
                        "report_id": report.report_id,
                        "sections": report.sections.len(),
                        "markdown_length": report.markdown_content.len(),
                    }),
                );
            }
            Err(e) => {
                task_manager_for_complete.fail_task(&task_id_clone, &e);
            }
        }
    });

    Ok(Json(serde_json::json!({
        "task_id": task_id,
        "message": "Report generation started",
    })))
}

/// Get report status
pub async fn get_status(
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
        "created_at": dict.created_at.to_rfc3339(),
    })))
}

/// Stream report generation progress via SSE
pub async fn stream_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let task = state.task_manager.get_task(&task_id);
    match task {
        Some(_) => {
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

/// Chat with a generated report
pub async fn chat_with_report(
    State(_state): State<AppState>,
    Json(_req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (axum::http::StatusCode, String)> {
    info!("Chat with report");

    let response = ChatResponse {
        message: "Report chat not yet fully implemented".to_string(),
        tools_used: vec![],
    };

    Ok(Json(response))
}

/// List reports
pub async fn list_reports(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    info!("Listing reports");
    
    let limit: usize = params.get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(50);

    let reports = state.report_store.list_reports(limit);
    
    Ok(Json(serde_json::json!({
        "success": true,
        "data": reports,
        "count": reports.len(),
    })))
}

/// Get report by ID
pub async fn get_report(
    State(state): State<AppState>,
    Path(report_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let report = state.report_store.get_report(&report_id)
        .ok_or((axum::http::StatusCode::NOT_FOUND, "Report not found".to_string()))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": report,
    })))
}

/// Get report by simulation ID
pub async fn get_report_by_simulation(
    State(state): State<AppState>,
    Path(simulation_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let report = state.report_store.get_report_by_simulation(&simulation_id)
        .ok_or((axum::http::StatusCode::NOT_FOUND, "No report found for this simulation".to_string()))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": report,
        "has_report": true,
    })))
}

/// Delete a report
pub async fn delete_report(
    State(state): State<AppState>,
    Path(report_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let success = state.report_store.delete_report(&report_id);
    
    if !success {
        return Err((axum::http::StatusCode::NOT_FOUND, "Report not found".to_string()));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Report {} deleted", report_id),
    })))
}

/// Get report sections
pub async fn get_report_sections(
    State(state): State<AppState>,
    Path(report_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let sections = state.report_store.get_sections(&report_id);
    let is_complete = state.report_store.is_report_complete(&report_id);

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "report_id": report_id,
            "sections": sections,
            "total_sections": sections.len(),
            "is_complete": is_complete,
        }
    })))
}

/// Get single section
pub async fn get_section(
    State(state): State<AppState>,
    Path((report_id, section_index)): Path<(String, usize)>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let section = state.report_store.get_section(&report_id, section_index)
        .ok_or((axum::http::StatusCode::NOT_FOUND, format!("Section {} not found", section_index)))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "filename": format!("section_{:02}.md", section_index),
            "section_index": section_index,
            "content": section,
        }
    })))
}

/// Check report status for a simulation
pub async fn check_report_status(
    State(state): State<AppState>,
    Path(simulation_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let report = state.report_store.get_report_by_simulation(&simulation_id);
    
    let has_report = report.is_some();
    let report_status = report.as_ref().map(|r| r.get("status").and_then(|v| v.as_str()).unwrap_or("unknown").to_string());
    let report_id = report.as_ref().and_then(|r| r.get("report_id").and_then(|v| v.as_str()).map(|s| s.to_string()));
    let interview_unlocked = has_report && report_status.as_deref() == Some("completed");

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "simulation_id": simulation_id,
            "has_report": has_report,
            "report_status": report_status,
            "report_id": report_id,
            "interview_unlocked": interview_unlocked,
        }
    })))
}
