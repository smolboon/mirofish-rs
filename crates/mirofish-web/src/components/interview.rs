//! Agent interview component (Leptos)

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api::ApiClient;

/// Interview page component
#[component]
pub fn InterviewPage() -> impl IntoView {
    let api_client = RwSignal::new(ApiClient::default());
    let selected_agent = RwSignal::new(0u64);
    let agents: RwSignal<Vec<serde_json::Value>> = RwSignal::new(vec![]);
    let chat_input = RwSignal::new(String::new());
    let chat_messages: RwSignal<Vec<(String, String)>> = RwSignal::new(vec![]);
    let is_loading = RwSignal::new(false);
    let status_message = RwSignal::new(String::new());

    // Load agents
    let load_agents = move || {
        let api = api_client.get();
        spawn_local(async move {
            is_loading.set(true);
            match api.list_agents().await {
                Ok(agent_list) => {
                    if let Some(data) = agent_list.get("agents").and_then(|v| v.as_array()) {
                        agents.set(data.clone());
                        status_message.set(format!("Loaded {} agents", data.len()));
                    }
                }
                Err(e) => status_message.set(format!("Failed to load agents: {}", e)),
            }
            is_loading.set(false);
        });
    };

    let send_message = move || {
        let api = api_client.get();
        let agent = selected_agent.get();
        let message = chat_input.get();

        if agent == 0 {
            status_message.set("Please select an agent first".to_string());
            return;
        }

        if message.trim().is_empty() {
            return;
        }

        chat_input.set(String::new());
        let mut messages = chat_messages.get();
        messages.push(("user".to_string(), message.clone()));
        chat_messages.set(messages);

        spawn_local(async move {
            is_loading.set(true);
            match api.interview_agent(agent, &message).await {
                Ok(response) => {
                    let mut messages = chat_messages.get();
                    if let Some(reply) = response.get("response").and_then(|v| v.as_str()) {
                        messages.push(("agent".to_string(), reply.to_string()));
                    } else {
                        messages.push(("agent".to_string(), "No response".to_string()));
                    }
                    chat_messages.set(messages);
                }
                Err(e) => {
                    let mut messages = chat_messages.get();
                    messages.push(("error".to_string(), format!("Error: {}", e)));
                    chat_messages.set(messages);
                }
            }
            is_loading.set(false);
        });
    };

    view! {
        <div class="interview-page">
            <h2>"Agent Interview"</h2>

            <div class="interview-controls">
                <button class="btn btn-primary" on:click=move |_| load_agents() disabled=is_loading>
                    "Load Agents"
                </button>
                <span class="status">{move || status_message.get()}</span>
            </div>

            <div class="agent-selector">
                <h3>"Select Agent"</h3>
                <select on:change=move |ev| {
                    let target = ev.target();
                    let value = target.and_then(|t| {
                        let el: web_sys::HtmlSelectElement = wasm_bindgen::JsCast::dyn_into(t).ok()?;
                        Some(el.value())
                    }).unwrap_or_default();
                    let agent_id: u64 = value.parse().unwrap_or(0);
                    selected_agent.set(agent_id);
                }>
                    <option value="">"-- Select an agent --"</option>
                    <For
                        each=move || agents.get()
                        key=|agent: &serde_json::Value| agent.get("id").and_then(|v| v.as_u64()).unwrap_or(0)
                        children=move |agent: serde_json::Value| {
                            let name = agent.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
                            let id = agent.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                            let value_str = format!("{}", id);
                            view! {
                                <option value=value_str>{name}</option>
                            }
                        }
                    />
                </select>
            </div>

            <div class="chat-container">
                <div class="chat-messages">
                    <For
                        each=move || chat_messages.get()
                        key=|(_, i): &(String, String)| i.clone()
                        children=move |(role, message): (String, String)| {
                            let class = match role.as_str() {
                                "user" => "chat-message user",
                                "agent" => "chat-message agent",
                                _ => "chat-message error",
                            };
                            view! {
                                <div class=class>
                                    <strong>{role.clone()}</strong>
                                    <p>{message.clone()}</p>
                                </div>
                            }
                        }
                    />
                </div>
                <div class="chat-input">
                    <input
                        type="text"
                        placeholder="Ask the agent a question..."
                        prop:value=move || chat_input.get()
                        on:input=move |ev| {
                            let target = ev.target();
                            let value = target.and_then(|t| {
                                let el: web_sys::HtmlInputElement = wasm_bindgen::JsCast::dyn_into(t).ok()?;
                                Some(el.value())
                            }).unwrap_or_default();
                            chat_input.set(value);
                        }
                    />
                    <button class="btn btn-primary" on:click=move |_| send_message() disabled=is_loading>
                        "Send"
                    </button>
                </div>
            </div>
        </div>
    }
}