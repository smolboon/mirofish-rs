//! Report planning and outline generation

use mirofish_core::{ReportOutline, ReportSection, ReportSectionOutline};
use mirofish_llm::LLMClient;
use mirofish_llm::prompts::{REPORT_PLAN_SYSTEM_PROMPT, REPORT_PLAN_USER_PROMPT_TEMPLATE};
use tracing::debug;

/// Generate a report outline using LLM
pub async fn generate_report_outline(
    llm: &LLMClient,
    simulation_requirement: &str,
    total_nodes: usize,
    total_edges: usize,
    entity_types: &[String],
    total_entities: usize,
    related_facts_json: &str,
) -> Result<ReportOutline, String> {
    debug!("Generating report outline");

    let entity_types_str = entity_types.join(", ");

    let prompt = REPORT_PLAN_USER_PROMPT_TEMPLATE
        .replace("{simulation_requirement}", simulation_requirement)
        .replace("{total_nodes}", &total_nodes.to_string())
        .replace("{total_edges}", &total_edges.to_string())
        .replace("{entity_types}", &entity_types_str)
        .replace("{total_entities}", &total_entities.to_string())
        .replace("{related_facts_json}", related_facts_json);

    let response: String = llm.chat_json(REPORT_PLAN_SYSTEM_PROMPT, &prompt).await
        .map_err(|e| format!("LLM error: {}", e))?;

    // Parse the JSON response into ReportOutline
    let outline_data: serde_json::Value = serde_json::from_str(&response)
        .map_err(|e| format!("Failed to parse outline JSON: {}", e))?;

    let title = outline_data
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Simulation Report")
        .to_string();

    let summary = outline_data
        .get("summary")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let sections_data = outline_data
        .get("sections")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut sections = Vec::new();
    for (i, section_data) in sections_data.iter().enumerate() {
        sections.push(ReportSectionOutline {
            title: section_data
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled")
                .to_string(),
            description: section_data
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        });
    }

    Ok(ReportOutline {
        title,
        summary,
        sections,
    })
}

/// Generate a single section using ReACT-enhanced research
pub async fn generate_section_with_research(
    llm: &LLMClient,
    section_title: &str,
    report_title: &str,
    report_summary: &str,
    simulation_requirement: &str,
    research_findings: &str,
) -> Result<String, String> {
    debug!("Generating section: {}", section_title);

    let system_prompt = format!(
        "You are writing a section of a future prediction report. \
        Report title: {}. Summary: {}. \
        Prediction scenario: {}. \
        Current section: {}. \
        You MUST use the available tools to gather data from the simulation. \
        Every claim must be backed by simulation evidence.",
        report_title, report_summary, simulation_requirement, section_title
    );

    let user_prompt = format!(
        "Write a comprehensive section titled '{}'.\n\nResearch findings:\n{}\n\n\
        Write in a professional report style with clear analysis and evidence.",
        section_title, research_findings
    );

    llm.chat(&system_prompt, &user_prompt).await
        .map_err(|e| format!("LLM error: {}", e))
}
