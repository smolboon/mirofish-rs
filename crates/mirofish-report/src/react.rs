//! ReACT (Reasoning + Acting) loop for report generation

use tracing::{debug, info};

use mirofish_llm::LLMClient;
use mirofish_llm::parsing::{parse_react_thought, parse_react_action, parse_final_answer};
use mirofish_graph::client::ZepClient;

use crate::tools::{ToolResult, insight_forge, panorama_search, quick_search, interview_agents};

/// Maximum ReACT iterations before giving up
const MAX_REACT_ITERATIONS: usize = 15;

/// Execute a ReACT loop for a specific query
pub async fn react_loop(
    llm: &LLMClient,
    zep: &ZepClient,
    graph_id: &str,
    system_prompt: &str,
    query: &str,
    context: &str,
) -> Result<String, String> {
    debug!("Starting ReACT loop for query: {}", query);

    let mut scratchpad = Vec::new();
    let mut iteration = 0;

    while iteration < MAX_REACT_ITERATIONS {
        iteration += 1;

        // Build the ReACT prompt
        let react_prompt = build_react_prompt(system_prompt, query, context, &scratchpad);

        // Get LLM response
        let response = llm.chat(system_prompt, &react_prompt).await
            .map_err(|e| format!("LLM error: {}", e))?;

        // Check for final answer first
        if let Some(answer) = parse_final_answer(&response) {
            info!("ReACT loop completed with final answer at iteration {}", iteration);
            return Ok(answer);
        }

        // Parse thought
        let thought = parse_react_thought(&response).unwrap_or_default();
        debug!("Thought: {}", thought);

        // Parse and execute action
        if let Some((tool_name, _tool_params)) = parse_react_action(&response) {
            debug!("Executing tool: {}", tool_name);

            let tool_result = execute_tool(llm, zep, graph_id, &tool_name, query).await?;
            debug!("Tool result: {} chars", tool_result.content.len());

            scratchpad.push(ReACTStep {
                thought,
                tool_name: tool_result.tool_name.clone(),
                tool_query: tool_result.query.clone(),
                observation: tool_result.content.clone(),
            });
        } else {
            // No action found, try to extract final answer from response
            debug!("No action found, using response as final answer");
            return Ok(response);
        }
    }

    Err("ReACT loop exceeded maximum iterations".to_string())
}

/// Execute a tool by name
async fn execute_tool(
    llm: &LLMClient,
    zep: &ZepClient,
    graph_id: &str,
    tool_name: &str,
    query: &str,
) -> Result<ToolResult, String> {
    match tool_name {
        "InsightForge" => insight_forge(zep, graph_id, query, 10).await,
        "PanoramaSearch" => panorama_search(zep, graph_id, query, 15).await,
        "QuickSearch" => quick_search(zep, graph_id, query, 5).await,
        "InterviewAgents" => {
            // For interview, we need agent name and question from query
            let parts: Vec<&str> = query.splitn(2, ':').collect();
            let agent_name = parts.first().unwrap_or(&"Unknown");
            let question = parts.get(1).unwrap_or(&"What happened?");
            interview_agents(llm, agent_name, question, "").await
        }
        _ => Err(format!("Unknown tool: {}", tool_name)),
    }
}

/// Build the ReACT prompt with scratchpad
fn build_react_prompt(
    system_prompt: &str,
    query: &str,
    context: &str,
    scratchpad: &[ReACTStep],
) -> String {
    let mut prompt = format!(
        "{}\n\nQuery: {}\n\nContext:\n{}\n\n",
        system_prompt, query, context
    );

    if !scratchpad.is_empty() {
        prompt.push_str("Previous steps:\n");
        for (i, step) in scratchpad.iter().enumerate() {
            prompt.push_str(&format!(
                "Step {}:\nThought: {}\nAction: {}\nObservation: {}\n\n",
                i + 1,
                step.thought,
                step.tool_name,
                step.observation.chars().take(500).collect::<String>()
            ));
        }
    }

    prompt.push_str("Now think step by step. Use Thought: ... Action: ... format.\n");
    prompt.push_str("When you have enough information, use Final Answer: ... to conclude.\n");

    prompt
}

/// A single ReACT step
#[derive(Debug, Clone)]
pub struct ReACTStep {
    pub thought: String,
    pub tool_name: String,
    pub tool_query: String,
    pub observation: String,
}