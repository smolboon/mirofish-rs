//! History database component (Leptos)

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api::ApiClient;

/// History database page component
#[component]
pub fn HistoryDatabase() -> impl IntoView {
    let api_client = RwSignal::new(ApiClient::default());
    let projects: RwSignal<Vec<serde_json::Value>> = RwSignal::new(vec![]);
    let is_loading = RwSignal::new(false);
    let status_message = RwSignal::new(String::new());
    let selected_project: RwSignal<Option<serde_json::Value>> = RwSignal::new(None);

    // Load projects
    let load_projects = move || {
        let api = api_client.get();
        spawn_local(async move {
            is_loading.set(true);
            match api.list_projects().await {
                Ok(result) => {
                    if let Some(data) = result.get("projects").and_then(|v| v.as_array()) {
                        projects.set(data.clone());
                        status_message.set(format!("Loaded {} projects", data.len()));
                    }
                }
                Err(e) => status_message.set(format!("Failed to load projects: {}", e)),
            }
            is_loading.set(false);
        });
    };

    // Delete project
    let delete_project = move |project_id: String| {
        let api = api_client.get();
        spawn_local(async move {
            is_loading.set(true);
            match api.delete_project(&project_id).await {
                Ok(_) => {
                    status_message.set("Project deleted".to_string());
                    // Reload projects
                    let api = api_client.get();
                    if let Ok(result) = api.list_projects().await {
                        if let Some(data) = result.get("projects").and_then(|v| v.as_array()) {
                            projects.set(data.clone());
                        }
                    }
                }
                Err(e) => status_message.set(format!("Failed to delete project: {}", e)),
            }
            is_loading.set(false);
        });
    };

    view! {
        <div class="history-database">
            <h2>"History Database"</h2>

            <div class="history-controls">
                <button class="btn btn-primary" on:click=move |_| load_projects() disabled=is_loading>
                    "Load Projects"
                </button>
                <span class="status">{move || status_message.get()}</span>
            </div>

            <div class="projects-list">
                <For
                    each=move || projects.get()
                    key=|project| project.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string()
                    children=move |project: serde_json::Value| {
                        let name = project.get("name").and_then(|v| v.as_str()).unwrap_or("Unnamed").to_string();
                        let id = project.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let created_at = project.get("created_at").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                        let delete_id = id.clone();
                        let project_clone = project.clone();

                        view! {
                            <div class="project-card">
                                <h3>{name}</h3>
                                <p>"ID: " {id.clone()}</p>
                                <p>"Created: " {created_at}</p>
                                <div class="project-actions">
                                    <button class="btn btn-secondary" on:click=move |_| {
                                        selected_project.set(Some(project_clone.clone()));
                                    }>
                                        "View Details"
                                    </button>
                                    <button class="btn btn-danger" on:click=move |_| delete_project(delete_id.clone()) disabled=is_loading>
                                        "Delete"
                                    </button>
                                </div>
                            </div>
                        }
                    }
                />
            </div>

            <Show when=move || selected_project.get().is_some() fallback=|| view! {}>
                <div class="project-details">
                    <h3>"Project Details"</h3>
                    <pre>{move || {
                        selected_project.get()
                            .map(|p| serde_json::to_string_pretty(&p).unwrap_or_default())
                            .unwrap_or_default()
                    }}</pre>
                    <button class="btn btn-secondary" on:click=move |_| selected_project.set(None)>
                        "Close"
                    </button>
                </div>
            </Show>
        </div>
    }
}