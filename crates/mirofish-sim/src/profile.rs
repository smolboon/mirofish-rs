//! Agent profile generation from Zep entities

use mirofish_core::{AgentProfile, Persona, Demographics, ActivityPattern};
use mirofish_graph::ZepEntity;
use mirofish_llm::LLMClient;
use mirofish_llm::prompts::{PROFILE_SYSTEM_PROMPT, PROFILE_USER_PROMPT_TEMPLATE};
use tracing::debug;

/// Generate agent profiles from Zep entities using LLM
pub async fn generate_profiles_from_entities(
    llm: &LLMClient,
    entities: &[ZepEntity],
    simulation_requirement: &str,
    use_llm: bool,
) -> Result<Vec<AgentProfile>, String> {
    let mut profiles = Vec::new();

    for (i, entity) in entities.iter().enumerate() {
        let profile = if use_llm {
            generate_profile_with_llm(llm, entity, simulation_requirement).await?
        } else {
            generate_profile_manual(entity, i)
        };
        profiles.push(profile);
    }

    Ok(profiles)
}

/// Generate a single profile using LLM
async fn generate_profile_with_llm(
    llm: &LLMClient,
    entity: &ZepEntity,
    simulation_requirement: &str,
) -> Result<AgentProfile, String> {
    debug!("Generating profile for entity: {}", entity.name);

    let attrs = entity.metadata
        .as_ref()
        .map(|m| m.to_string())
        .unwrap_or_default();

    let prompt = PROFILE_USER_PROMPT_TEMPLATE
        .replace("{entity_name}", &entity.name)
        .replace("{entity_type}", &entity.entity_type)
        .replace("{entity_description}", entity.description.as_deref().unwrap_or(""))
        .replace("{entity_attributes}", &attrs)
        .replace("{entity_relations}", "")
        .replace("{simulation_requirement}", simulation_requirement);

    let response = llm.chat_json(PROFILE_SYSTEM_PROMPT, &prompt).await
        .map_err(|e| format!("LLM error: {}", e))?;

    // Parse the response into AgentProfile
    // The LLM should return JSON matching our structure
    let profile_data: serde_json::Value = response;

    Ok(AgentProfile {
        agent_id: 0, // Will be set during simulation setup
        name: profile_data.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&entity.name)
            .to_string(),
        username: profile_data.get("username")
            .and_then(|v| v.as_str())
            .unwrap_or(&format!("user_{}", entity.uuid.chars().take(8).collect::<String>()))
            .to_string(),
        bio: profile_data.get("bio")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| entity.summary.as_deref().unwrap_or(""))
            .to_string(),
        persona: Persona {
            personality_traits: profile_data.get("persona")
                .and_then(|p| p.get("personality_traits"))
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default(),
            interests: profile_data.get("persona")
                .and_then(|p| p.get("interests"))
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default(),
            behavioral_tendencies: profile_data.get("persona")
                .and_then(|p| p.get("behavioral_tendencies"))
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default(),
            communication_style: profile_data.get("persona")
                .and_then(|p| p.get("communication_style"))
                .and_then(|v| v.as_str())
                .unwrap_or("neutral")
                .to_string(),
            stance_on_topic: profile_data.get("persona")
                .and_then(|p| p.get("stance_on_topic"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        },
        demographics: Demographics {
            age_group: profile_data.get("demographics")
                .and_then(|d| d.get("age_group"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            gender: profile_data.get("demographics")
                .and_then(|d| d.get("gender"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            occupation: profile_data.get("demographics")
                .and_then(|d| d.get("occupation"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            location: profile_data.get("demographics")
                .and_then(|d| d.get("location"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            education: profile_data.get("demographics")
                .and_then(|d| d.get("education"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
        },
        activity_pattern: ActivityPattern {
            activity_level: profile_data.get("activity_pattern")
                .and_then(|a| a.get("activity_level"))
                .and_then(|v| v.as_str())
                .unwrap_or("medium")
                .to_string(),
            posting_frequency: profile_data.get("activity_pattern")
                .and_then(|a| a.get("posting_frequency"))
                .and_then(|v| v.as_str())
                .unwrap_or("normal")
                .to_string(),
            peak_hours: profile_data.get("activity_pattern")
                .and_then(|a| a.get("peak_hours"))
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_else(|| vec![9, 12, 18, 21]),
            preferred_topics: profile_data.get("activity_pattern")
                .and_then(|a| a.get("preferred_topics"))
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default(),
        },
        initial_beliefs: Vec::new(),
        social_network: Vec::new(),
    })
}

/// Generate a manual profile without LLM (fallback)
fn generate_profile_manual(entity: &ZepEntity, index: usize) -> AgentProfile {
    AgentProfile {
        agent_id: index,
        name: entity.name.clone(),
        username: format!("user_{}", entity.uuid.chars().take(8).collect::<String>()),
        bio: entity.summary.clone().unwrap_or_default(),
        persona: Persona {
            personality_traits: Default::default(),
            interests: Vec::new(),
            behavioral_tendencies: Default::default(),
            communication_style: "neutral".to_string(),
            stance_on_topic: "".to_string(),
        },
        demographics: Demographics {
            age_group: "unknown".to_string(),
            gender: "unknown".to_string(),
            occupation: entity.entity_type.clone(),
            location: "unknown".to_string(),
            education: "unknown".to_string(),
        },
        activity_pattern: ActivityPattern {
            activity_level: "medium".to_string(),
            posting_frequency: "normal".to_string(),
            peak_hours: vec![9, 12, 18, 21],
            preferred_topics: Vec::new(),
        },
        initial_beliefs: Vec::new(),
        social_network: Vec::new(),
    }
}