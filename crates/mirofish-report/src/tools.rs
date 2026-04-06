//! Report agent tools - search strategies for gathering simulation data

use mirofish_graph::client::{ZepClient, SearchResult};
use mirofish_llm::LLMClient;
use tracing::debug;

/// Tool result
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub tool_name: String,
    pub query: String,
    pub content: String,
    pub entities: Vec<String>,
    pub facts: Vec<String>,
}

/// InsightForge - Deep, multi-dimensional analysis tool
pub async fn insight_forge(
    zep: &ZepClient,
    graph_id: &str,
    query: &str,
    limit: usize,
) -> Result<ToolResult, String> {
    debug!("InsightForge query: {}", query);

    // Decompose query into sub-queries for multi-dimensional search
    let sub_queries = decompose_query(query);

    let mut all_results = Vec::new();
    for sub_query in &sub_queries {
        match zep.search_graph(graph_id, sub_query, limit).await {
            Ok(results) => all_results.extend(results),
            Err(e) => debug!("InsightForge sub-query failed: {}", e),
        }
    }

    // Deduplicate and format
    let mut entities = Vec::new();
    let mut facts = Vec::new();

    for result in &all_results {
        if let Some(entity) = &result.entity {
            entities.push(entity.name.clone());
        }
        if let Some(f) = &result.facts {
            facts.extend(f.clone());
        }
    }

    entities.dedup();
    facts.dedup();

    let content = format!(
        "InsightForge Results for '{}':\n\nEntities found: {}\nFacts: {}",
        query,
        entities.join(", "),
        facts.join("\n")
    );

    Ok(ToolResult {
        tool_name: "InsightForge".to_string(),
        query: query.to_string(),
        content,
        entities,
        facts,
    })
}

/// PanoramaSearch - Broad, full-picture view
pub async fn panorama_search(
    zep: &ZepClient,
    graph_id: &str,
    query: &str,
    limit: usize,
) -> Result<ToolResult, String> {
    debug!("PanoramaSearch query: {}", query);

    let results = zep
        .search_graph(graph_id, query, limit)
        .await
        .map_err(|e| e.to_string())?;

    let mut entities = Vec::new();
    let mut facts = Vec::new();

    for result in &results {
        if let Some(entity) = &result.entity {
            entities.push(entity.name.clone());
        }
        if let Some(f) = &result.facts {
            facts.extend(f.clone());
        }
    }

    entities.dedup();
    facts.dedup();

    let content = format!(
        "PanoramaSearch Results for '{}':\n\nEntities: {}\nFacts: {}",
        query,
        entities.join(", "),
        facts.join("\n")
    );

    Ok(ToolResult {
        tool_name: "PanoramaSearch".to_string(),
        query: query.to_string(),
        content,
        entities,
        facts,
    })
}

/// QuickSearch - Lightweight, fast lookup
pub async fn quick_search(
    zep: &ZepClient,
    graph_id: &str,
    query: &str,
    limit: usize,
) -> Result<ToolResult, String> {
    debug!("QuickSearch query: {}", query);

    let results = zep
        .search_graph(graph_id, query, limit)
        .await
        .map_err(|e| e.to_string())?;

    let mut entities = Vec::new();
    let mut facts = Vec::new();

    for result in &results {
        if let Some(entity) = &result.entity {
            entities.push(entity.name.clone());
        }
        if let Some(f) = &result.facts {
            facts.extend(f.clone());
        }
    }

    let content = format!(
        "QuickSearch Results for '{}':\n{}",
        query,
        facts.join("\n")
    );

    Ok(ToolResult {
        tool_name: "QuickSearch".to_string(),
        query: query.to_string(),
        content,
        entities,
        facts,
    })
}

/// InterviewAgents - Query specific agents about their experiences
pub async fn interview_agents(
    llm: &LLMClient,
    agent_name: &str,
    question: &str,
    agent_context: &str,
) -> Result<ToolResult, String> {
    debug!("InterviewAgents: {} asks '{}'", agent_name, question);

    let system_prompt = format!(
        "You are {}. Answer the question based on your simulated experiences and beliefs.\n\nContext: {}",
        agent_name, agent_context
    );

    let response = llm.chat(&system_prompt, question).await
        .map_err(|e| format!("LLM error: {}", e))?;

    let content = format!(
        "Interview with {}:\nQ: {}\nA: {}",
        agent_name, question, response
    );

    Ok(ToolResult {
        tool_name: "InterviewAgents".to_string(),
        query: format!("{}: {}", agent_name, question),
        content,
        entities: vec![agent_name.to_string()],
        facts: vec![response],
    })
}

/// Decompose a complex query into multiple sub-queries
fn decompose_query(query: &str) -> Vec<String> {
    let mut sub_queries = vec![query.to_string()];

    // Add variations for broader coverage
    let words: Vec<&str> = query.split_whitespace().collect();
    if words.len() > 3 {
        // Take first half
        sub_queries.push(words[..words.len() / 2].join(" "));
        // Take second half
        sub_queries.push(words[words.len() / 2..].join(" "));
    }

    sub_queries
}