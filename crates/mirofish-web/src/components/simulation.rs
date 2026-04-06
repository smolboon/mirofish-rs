//! Simulation control component (Leptos)

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use crate::api::ApiClient;

/// Simulation status
#[derive(Debug, Clone, Default)]
pub struct SimulationStatus {
    pub posts: u64,
    pub comments: u64,
    pub actions: u64,
    pub round: u64,
    pub progress_percent: f64,
    pub is_running: bool,
}

/// Simulation control component
#[component]
pub fn SimulationPage() -> impl IntoView {
    let api_client = RwSignal::new(ApiClient::default());
    let simulation_id = RwSignal::new(String::new());
    let project_id = RwSignal::new(String::new());
    let graph_id = RwSignal::new(String::new());
    let enable_twitter = RwSignal::new(true);
    let enable_reddit = RwSignal::new(true);
    let status = RwSignal::new(SimulationStatus::default());
    let is_loading = RwSignal::new(false);
    let message = RwSignal::new(String::new());

    let start_simulation = move || {
        let api = api_client.get();
        let proj_id = project_id.get();
        let graph = graph_id.get();
        let twitter = enable_twitter.get();
        let reddit = enable_reddit.get();

        spawn_local(async move {
            is_loading.set(true);
            message.set("Creating simulation...".to_string());

            match api.create_simulation(&proj_id, &graph, twitter, reddit).await {
                Ok(result) => {
                    if let Some(sim_id) = result.get("simulation_id").and_then(|v| v.as_str()) {
                        simulation_id.set(sim_id.to_string());
                        message.set("Simulation created. Preparing...".to_string());

                        match api.prepare_simulation(sim_id, &graph, "Simulation", "", twitter, reddit).await {
                            Ok(_) => {
                                message.set("Simulation prepared. Starting...".to_string());
                                let sim_id = simulation_id.get();
                                let proj_id = project_id.get();
                                let graph = graph_id.get();
                                let twitter = enable_twitter.get();
                                let reddit = enable_reddit.get();
                                match api.start_simulation(
                                    &sim_id, &proj_id, &graph, twitter, reddit,
                                    &serde_json::json!({}), &serde_json::json!({})
                                ).await {
                                    Ok(_) => {
                                        message.set("Simulation started!".to_string());
                                        status.update(|s| s.is_running = true);
                                    }
                                    Err(e) => message.set(format!("Failed to start: {}", e)),
                                }
                            }
                            Err(e) => message.set(format!("Failed to prepare: {}", e)),
                        }
                    }
                }
                Err(e) => message.set(format!("Failed to create: {}", e)),
            }
            is_loading.set(false);
        });
    };

    let stop_simulation = move || {
        let api = api_client.get();
        let sim_id = simulation_id.get();

        spawn_local(async move {
            is_loading.set(true);
            match api.get_simulation_status(&sim_id).await {
                Ok(status_data) => {
                    if let Some(state) = status_data.get("status").and_then(|v| v.as_str()) {
                        message.set(format!("Status: {}", state));
                        if state == "completed" || state == "failed" {
                            status.update(|s| s.is_running = false);
                        }
                    }
                }
                Err(e) => message.set(format!("Failed to get status: {}", e)),
            }
            is_loading.set(false);
        });
    };

    view! {
        <div class="simulation-page">
            <h2>"Simulation Control"</h2>

            <div class="simulation-controls">
                <div class="control-group">
                    <label>
                        <input type="checkbox" checked=enable_twitter
                            on:change=move |ev| {
                                let target = ev.target();
                                let checked = target.and_then(|t| {
                                    let el: web_sys::HtmlInputElement = wasm_bindgen::JsCast::dyn_into(t).ok()?;
                                    Some(el.checked())
                                }).unwrap_or(true);
                                enable_twitter.set(checked);
                            }
                        />
                        "Enable Twitter"
                    </label>
                </div>
                <div class="control-group">
                    <label>
                        <input type="checkbox" checked=enable_reddit
                            on:change=move |ev| {
                                let target = ev.target();
                                let checked = target.and_then(|t| {
                                    let el: web_sys::HtmlInputElement = wasm_bindgen::JsCast::dyn_into(t).ok()?;
                                    Some(el.checked())
                                }).unwrap_or(true);
                                enable_reddit.set(checked);
                            }
                        />
                        "Enable Reddit"
                    </label>
                </div>
            </div>

            <div class="simulation-actions">
                <button class="btn btn-success" on:click=move |_| start_simulation() disabled=is_loading>
                    "Start Simulation"
                </button>
                <button class="btn btn-danger" on:click=move |_| stop_simulation() disabled=is_loading>
                    "Check Status"
                </button>
            </div>

            <div class="simulation-status">
                <h3>"Simulation Progress"</h3>
                <div class="status-display">
                    <p>"Simulation ID: " {move || simulation_id.get()}</p>
                    <p>"Status: " {move || message.get()}</p>
                </div>
            </div>
        </div>
    }
}