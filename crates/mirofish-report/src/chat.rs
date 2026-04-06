//! Chat interface for post-report interaction

use mirofish_core::{ChatMessage, ChatRequest, ChatResponse, Report};
use mirofish_llm::LLMClient;
use mirofish_graph::client::ZepClient;
use tracing::debug;

/// Chat with the report using ReACT-enhanced responses
pub async fn chat_with_report(
    llm: &LLMClient,
    zep: &ZepClient,
    graph_id: &str,
    report: &Report,
    request: &ChatRequest,
) -> Result<ChatResponse, String> {
    debug!("Chat with report: {}", request.message);

    // Build system prompt with report context
    let system_prompt = build_chat_system_prompt(report);

    // Build user prompt with chat history
    let user_prompt = build_chat_prompt(&request.history, &request.message);

    // Get LLM response
    let response = llm.chat(&system_prompt, &user_prompt).await
        .map_err(|e| format!("LLM error: {}", e))?;

    // Determine which tools were used (based on response content)
    let tools_used = detect_tools_used(&response);

    Ok(ChatResponse {
        message: response,
        tools_used,
    })
}

/// Build system prompt for chat
fn build_chat_system_prompt(report: &Report) -> String {
    let outline_info = if let Some(outline) = &report.outline {
        format!(
            "Report: {}\nSummary: {}\nSections: {}",
            outline.title,
            outline.summary,
            outline.sections.iter().map(|s| s.title.as_str()).collect::<Vec<_>>().join(", ")
        )
    } else {
        "Report outline not available.".to_string()
    };

    let sections_info = report.sections.iter()
        .map(|s| format!("- {}: {}", s.title, s.content.chars().take(200).collect::<String>()))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "You are an AI assistant with full knowledge of a simulation report.\n\n\
        {}\n\n\
        Report sections:\n{}\n\n\
        You can use tools to query the simulation graph for more details. \
        Answer questions about the simulation and its predictions accurately.",
        outline_info, sections_info
    )
}

/// Build user prompt with chat history
fn build_chat_prompt(history: &[ChatMessage], message: &str) -> String {
    let mut prompt = String::new();

    for msg in history.iter().take(10) {
        prompt.push_str(&format!("{}: {}\n", msg.role, msg.content));
    }

    prompt.push_str(&format!("user: {}\nassistant:", message));
    prompt
}

/// Detect which tools were referenced in the response
fn detect_tools_used(response: &str) -> Vec<String> {
    let mut tools = Vec::new();

    if response.contains("InsightForge") || response.contains("deep analysis") {
        tools.push("InsightForge".to_string());
    }
    if response.contains("PanoramaSearch") || response.contains("broad view") {
        tools.push("PanoramaSearch".to_string());
    }
    if response.contains("QuickSearch") || response.contains("quick lookup") {
        tools.push("QuickSearch".to_string());
    }
    if response.contains("InterviewAgents") || response.contains("agent said") {
        tools.push("InterviewAgents".to_string());
    }

    tools
}