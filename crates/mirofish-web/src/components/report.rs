//! Report generation and viewing component (Leptos)

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api::ApiClient;

/// Simple markdown to HTML conversion
fn markdown_to_html(markdown: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;
    let mut in_list = false;

    for line in markdown.lines() {
        if line.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>");
                in_code_block = false;
            } else {
                html.push_str("<pre><code>");
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            html.push_str(line);
            html.push('\n');
            continue;
        }

        if line.starts_with("# ") {
            if in_list { html.push_str("</ul>"); in_list = false; }
            html.push_str(&format!("<h1>{}</h1>", &line[2..]));
        } else if line.starts_with("## ") {
            if in_list { html.push_str("</ul>"); in_list = false; }
            html.push_str(&format!("<h2>{}</h2>", &line[3..]));
        } else if line.starts_with("### ") {
            if in_list { html.push_str("</ul>"); in_list = false; }
            html.push_str(&format!("<h3>{}</h3>", &line[4..]));
        } else if line.starts_with("- ") {
            if !in_list { html.push_str("<ul>"); in_list = true; }
            html.push_str(&format!("<li>{}</li>", &line[2..]));
        } else if line.is_empty() {
            if in_list { html.push_str("</ul>"); in_list = false; }
            html.push_str("<br/>");
        } else {
            if in_list { html.push_str("</ul>"); in_list = false; }
            html.push_str(&format!("<p>{}</p>", line));
        }
    }

    if in_list {
        html.push_str("</ul>");
    }

    html
}

/// Report page component
#[component]
pub fn ReportPage() -> impl IntoView {
    let api_client = RwSignal::new(ApiClient::default());
    let simulation_id = RwSignal::new(String::new());
    let graph_id = RwSignal::new(String::new());
    let simulation_requirement = RwSignal::new(String::new());
    let report_content = RwSignal::new(String::new());
    let is_generating = RwSignal::new(false);
    let has_report = RwSignal::new(false);
    let status_message = RwSignal::new(String::new());

    let generate_report = move || {
        let api = api_client.get();
        let sim_id = simulation_id.get();
        let graph = graph_id.get();
        let requirement = simulation_requirement.get();

        spawn_local(async move {
            is_generating.set(true);
            status_message.set("Generating report...".to_string());

            match api.generate_report(&sim_id, &graph, &requirement).await {
                Ok(result) => {
                    if let Some(task_id) = result.get("task_id").and_then(|v| v.as_str()) {
                        status_message.set(format!("Report generation started. Task: {}", task_id));
                        has_report.set(true);
                    }
                }
                Err(e) => status_message.set(format!("Failed to generate report: {}", e)),
            }
            is_generating.set(false);
        });
    };

    let report_html = move || markdown_to_html(&report_content.get());

    view! {
        <div class="report-page">
            <h2>"Simulation Report"</h2>

            <div class="report-actions">
                <button class="btn btn-primary" on:click=move |_| generate_report() disabled=is_generating>
                    "Generate Report"
                </button>
            </div>

            <div class="report-status">
                <p>{move || status_message.get()}</p>
            </div>

            <Show when=move || has_report.get() fallback=|| view! { <p>"No report generated yet"</p> }>
                <div class="report-content">
                    <div class="report-header">
                        <h3>"Simulation Report"</h3>
                    </div>
                    <div class="report-body" inner_html=report_html></div>
                </div>
            </Show>
        </div>
    }
}