//! Prompt templates for LLM interactions
//!
//! All prompts used in the system are defined here as constants.
//! Templates use {{variable}} syntax and are filled at runtime.

// ── Ontology Generation Prompts ──

pub const ONTOLOGY_SYSTEM_PROMPT: &str = r#"You are an expert knowledge graph designer. Analyze the provided documents and create an ontology (entity types and relation types) that captures the key concepts and their relationships for building a simulation knowledge graph.

Output valid JSON with:
- entity_types: array of {name, description, attributes: [{name, description, data_type}]}
- edge_types: array of {name, description, source_types: [...], target_types: [...]}
- analysis_summary: brief summary of the document content"#;

pub const ONTOLOGY_USER_PROMPT_TEMPLATE: &str = r#"Documents to analyze:

{document_texts}

Simulation requirement: {simulation_requirement}

{additional_context}

Create an ontology that will support simulating the described scenario."#;

// ── Agent Profile Generation Prompts ──

pub const PROFILE_SYSTEM_PROMPT: &str = "You are an expert at creating realistic agent personas for social simulations.";

pub const PROFILE_USER_PROMPT_TEMPLATE: &str = r#"Generate a detailed agent profile based on:

Entity: {entity_name}
Entity Type: {entity_type}
Description: {entity_description}
Attributes: {entity_attributes}
Relations: {entity_relations}

Simulation requirement: {simulation_requirement}

Create a complete profile with:
- name, username, bio
- personality traits
- demographics
- activity patterns
- stance on the topic"#;

// ── Simulation Config Generation Prompts ──

pub const SIM_CONFIG_SYSTEM_PROMPT: &str = "You are an expert simulation designer. Analyze the simulation requirements and generate optimal configuration for multi-agent social simulation.";

pub const SIM_CONFIG_USER_PROMPT_TEMPLATE: &str = r#"Simulation requirement: {simulation_requirement}

Document context: {document_text}

Number of agents: {entities_count}
Agent types: {entity_types}

Generate a complete simulation configuration with:
- Time settings (total hours, minutes per round)
- Activity patterns for each agent
- Initial posts and hot topics
- Platform-specific settings"#;

// ── Report Generation Prompts ──

pub const REPORT_PLAN_SYSTEM_PROMPT: &str = r#"You are an expert report writer with a "God's-eye view" of a simulated world. You can see everything that happened in the simulation. Plan a report that answers: what happened when we injected specific variables into this simulated world?"#;

pub const REPORT_PLAN_USER_PROMPT_TEMPLATE: &str = r#"Simulation requirement: {simulation_requirement}
World scale: {total_nodes} nodes, {total_edges} edges
Entity types: {entity_types}
Active agents: {total_entities}

Sample facts from simulation:
{related_facts_json}

Design a report outline (2-5 chapters) that reveals the key predictions and insights from this simulation."#;

pub const REPORT_SECTION_SYSTEM_PROMPT_TEMPLATE: &str = r#"You are writing a section of a future prediction report. Report title: {report_title}. Summary: {report_summary}. Prediction scenario: {simulation_requirement}. Current section: {section_title}.

You MUST use the available tools to gather data from the simulation. Every claim must be backed by simulation evidence."#;

// ── Interview Prompts ──

pub const INTERVIEW_SYSTEM_PROMPT: &str = "You are conducting an interview with a simulated agent. Respond in character based on the agent's persona, memories, and past actions.";