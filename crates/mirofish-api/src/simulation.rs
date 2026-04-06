//! Simulation API handlers

use axum::{
    extract::{Path, State},
    Json,
};
use tracing::info;

use mirofish_core::{
    Simulation, CreateSimulationRequest, PrepareSimulationRequest,
    StartSimulationRequest, SimulationStatusResponse,
};

use crate::state::AppState;

/// Create a new simulation
pub async fn create_simulation(
    State(state): State<AppState>,
    Json(req): Json<CreateSimulationRequest>,
) -> Result<Json<SimulationStatusResponse>, (axum::http::StatusCode, String)> {
    info!("Creating simulation for project: {}", req.project_id);

    let simulation = Simulation::new(
        &req.project_id,
        &req.graph_id,
        req.enable_twitter,
        req.enable_reddit,
    );

    let sim_id = simulation.simulation_id.clone();
    let status = simulation.status.clone();
    let current_round = simulation.current_round;
    let total_rounds = simulation.total_rounds;
    let error = simulation.error.clone();
    let started_at = simulation.started_at;
    let completed_at = simulation.completed_at;

    // Store simulation in state
    {
        let mut simulations = state.simulations.write().await;
        simulations.insert(sim_id.clone(), simulation);
    }

    Ok(Json(SimulationStatusResponse {
        simulation_id: sim_id,
        status,
        current_round,
        total_rounds,
        error,
        started_at,
        completed_at,
    }))
}

/// Prepare simulation (generate profiles and config)
pub async fn prepare_simulation(
    State(state): State<AppState>,
    Json(req): Json<PrepareSimulationRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    info!("Preparing simulation: {}", req.simulation_id);

    let task_id = state.task_manager.create_task(
        "simulation_prepare",
        serde_json::json!({
            "simulation_id": req.simulation_id,
        }),
    );

    let llm = state.llm.clone();
    let zep = state.zep.clone();
    let task_manager = state.task_manager.clone();
    let task_id_clone = task_id.clone();
    let graph_id = req.graph_id.clone();
    let simulation_requirement = req.simulation_requirement.clone();
    let document_text = req.document_text.clone();
    let enable_twitter = req.enable_twitter;
    let enable_reddit = req.enable_reddit;

    tokio::spawn(async move {
        // Step 1: Get entities from Zep
        let entities = match zep.get_entities(&graph_id, None).await {
            Ok(e) => e,
            Err(e) => {
                task_manager.fail_task(&task_id_clone, &e.to_string());
                return;
            }
        };

        task_manager.update_task(
            &task_id_clone,
            None,
            Some(30),
            Some("Entities fetched, generating profiles..."),
            None,
        );

        // Step 2: Generate profiles
        let profiles = match mirofish_sim::generate_profiles_from_entities(
            &llm,
            &entities,
            simulation_requirement.as_deref().unwrap_or(""),
            true,
        ).await {
            Ok(p) => p,
            Err(e) => {
                task_manager.fail_task(&task_id_clone, &e);
                return;
            }
        };

        task_manager.update_task(
            &task_id_clone,
            None,
            Some(60),
            Some("Profiles generated, generating config..."),
            None,
        );

        // Step 3: Generate simulation config
        let sim_config = match mirofish_sim::generate_simulation_config(
            &llm,
            simulation_requirement.as_deref().unwrap_or(""),
            &document_text,
            &entities,
            enable_twitter,
            enable_reddit,
        ).await {
            Ok(c) => c,
            Err(e) => {
                task_manager.fail_task(&task_id_clone, &e);
                return;
            }
        };

        task_manager.complete_task(
            &task_id_clone,
            serde_json::json!({
                "profiles_count": profiles.len(),
                "config": sim_config,
            }),
        );
    });

    Ok(Json(serde_json::json!({
        "task_id": task_id,
        "message": "Simulation preparation started",
    })))
}

/// Start simulation
pub async fn start_simulation(
    State(state): State<AppState>,
    Json(req): Json<StartSimulationRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    info!("Starting simulation: {}", req.simulation_id);

    let task_id = state.task_manager.create_task(
        "simulation_run",
        serde_json::json!({
            "simulation_id": req.simulation_id,
        }),
    );

    // Get simulation from state
    let simulation = {
        let simulations = state.simulations.read().await;
        simulations.get(&req.simulation_id).cloned()
    };

    let simulation = match simulation {
        Some(s) => s,
        None => {
            return Err((axum::http::StatusCode::NOT_FOUND, "Simulation not found".to_string()));
        }
    };

    let config = req.simulation_config.clone();
    let profiles = req.profiles.clone();

    // Convert core types to sim types (they use the same types from mirofish_core)
    let engine = mirofish_sim::SimulationEngine::new(
        simulation,
        config,
        profiles,
    );

    let llm = state.llm.clone();
    let task_manager = state.task_manager.clone();
    let task_id_clone = task_id.clone();
    let sim_id = req.simulation_id.clone();

    // Store engine in state for stop/status operations
    {
        let mut engines = state.simulation_engines.write().await;
        engines.insert(req.simulation_id.clone(), engine);
    }

    tokio::spawn(async move {
        let mut engines = state.simulation_engines.write().await;
        if let Some(engine) = engines.get_mut(&sim_id) {
            match engine.run(&llm, &task_manager, &task_id_clone).await {
                Ok(_) => {
                    task_manager.complete_task(
                        &task_id_clone,
                        serde_json::json!({
                            "posts": engine.posts.len(),
                            "comments": engine.comments.len(),
                            "actions": engine.total_actions,
                        }),
                    );
                }
                Err(e) => {
                    task_manager.fail_task(&task_id_clone, &e);
                }
            }
        }
    });

    Ok(Json(serde_json::json!({
        "task_id": task_id,
        "message": "Simulation started",
    })))
}

/// Stop simulation
pub async fn stop_simulation(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let sim_id = req.get("simulation_id")
        .and_then(|v| v.as_str())
        .ok_or((axum::http::StatusCode::BAD_REQUEST, "Missing simulation_id".to_string()))?;

    let mut engines = state.simulation_engines.write().await;
    if let Some(engine) = engines.get_mut(sim_id) {
        engine.stop();
        Ok(Json(serde_json::json!({ "message": "Simulation stopped" })))
    } else {
        Err((axum::http::StatusCode::NOT_FOUND, "Simulation not found".to_string()))
    }
}

/// Get simulation status
pub async fn get_status(
    State(state): State<AppState>,
    Path(sim_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let engines = state.simulation_engines.read().await;
    if let Some(engine) = engines.get(&sim_id) {
        let run_state = engine.get_run_state();
        Ok(Json(serde_json::to_value(&run_state).unwrap_or_default()))
    } else {
        Err((axum::http::StatusCode::NOT_FOUND, "Simulation not found".to_string()))
    }
}

/// Stream simulation progress via SSE
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

/// Interview an agent
pub async fn interview_agent(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let message = req.get("message")
        .and_then(|v| v.as_str())
        .ok_or((axum::http::StatusCode::BAD_REQUEST, "Missing message".to_string()))?;

    let agent_id = req.get("agent_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    // Use default profile for now, in production would look up agent from simulation
    let agent_profile = mirofish_core::AgentProfile::default_for_interview();
    let session = mirofish_sim::InterviewSession::new(
        agent_id,
        agent_profile,
        vec![],
        vec![],
        vec![],
    );

    // Generate interview response
    let response = mirofish_sim::generate_interview_response(
        &state.llm,
        &session,
        message,
    ).await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(serde_json::json!({
        "response": response,
    })))
}

/// List available agents for interview
pub async fn list_agents(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    Ok(Json(serde_json::json!({ "agents": [] })))
}
