//! Simulation configuration generation using LLM

use chrono::Utc;
use tracing::debug;

use mirofish_core::{
    AppConfig, AgentConfig, EventConfig, HotTopic, InitialPost, PlatformConfig,
    PlatformDetailConfig, SimulationConfig, TimeConfig,
};
use mirofish_graph::ZepEntity;
use mirofish_llm::{LLMClient, prompts::*};

/// Generate simulation configuration from entities and requirements using LLM
pub async fn generate_simulation_config(
    llm: &LLMClient,
    simulation_requirement: &str,
    document_text: &str,
    entities: &[ZepEntity],
    enable_twitter: bool,
    enable_reddit: bool,
) -> Result<SimulationConfig, String> {
    debug!("Generating simulation config for {} entities", entities.len());

    let entity_types: Vec<&str> = entities.iter().map(|e| e.entity_type.as_str()).collect();
    let unique_types: Vec<&str> = entity_types.iter().copied().collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let entity_types_str = unique_types.join(", ");
    let entities_count = entities.len();

    let prompt = SIM_CONFIG_USER_PROMPT_TEMPLATE
        .replace("{simulation_requirement}", simulation_requirement)
        .replace("{document_text}", &document_text.chars().take(5000).collect::<String>())
        .replace("{entities_count}", &entities_count.to_string())
        .replace("{entity_types}", &entity_types_str);

    let response: String = llm.chat_json(SIM_CONFIG_SYSTEM_PROMPT, &prompt).await
        .map_err(|e| format!("LLM error: {}", e))?;

    let config_data: serde_json::Value = serde_json::from_str(&response)
        .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

    // Parse time config
    let time_config = parse_time_config(&config_data, entities_count);

    // Parse agent configs
    let agent_configs = parse_agent_configs(&config_data, entities);

    // Parse event config
    let event_config = parse_event_config(&config_data);

    // Parse platform config
    let platform_config = PlatformConfig {
        twitter_config: if enable_twitter {
            Some(PlatformDetailConfig {
                agent_count: entities_count,
                subreddit: "twitter".to_string(),
                topic: simulation_requirement.chars().take(50).collect(),
            })
        } else {
            None
        },
        reddit_config: if enable_reddit {
            Some(PlatformDetailConfig {
                agent_count: entities_count,
                subreddit: "simulation".to_string(),
                topic: simulation_requirement.chars().take(50).collect(),
            })
        } else {
            None
        },
    };

    // Extract reasoning
    let generation_reasoning = config_data
        .get("generation_reasoning")
        .and_then(|v| v.as_str())
        .unwrap_or("Configuration generated based on entity analysis and simulation requirements.")
        .to_string();

    Ok(SimulationConfig {
        time_config,
        agent_configs,
        event_config,
        platform_config,
        generation_reasoning,
        generated_at: Some(Utc::now()),
        llm_model: None,
    })
}

fn parse_time_config(data: &serde_json::Value, entities_count: usize) -> TimeConfig {
    let time_data = data.get("time_config");

    let total_hours = time_data
        .and_then(|t| t.get("total_simulation_hours"))
        .and_then(|v| v.as_u64())
        .unwrap_or(72) as u32; // Default: 3 days

    let minutes_per_round = time_data
        .and_then(|t| t.get("minutes_per_round"))
        .and_then(|v| v.as_u64())
        .unwrap_or(30) as u32;

    let total_rounds = if minutes_per_round > 0 {
        (total_hours * 60) / minutes_per_round
    } else {
        144 // Default: 144 rounds
    };

    // More entities = more active hours to simulate social dynamics
    let peak_hours = if entities_count > 50 {
        vec![8, 9, 10, 11, 12, 13, 14, 18, 19, 20, 21, 22]
    } else {
        vec![9, 12, 18, 21]
    };

    let off_peak_hours: Vec<usize> = (0..24).filter(|h| !peak_hours.contains(h)).collect();

    TimeConfig {
        total_simulation_hours: total_hours,
        minutes_per_round,
        peak_hours,
        off_peak_hours,
        peak_activity_multiplier: 2.0,
    }
}

fn parse_agent_configs(
    data: &serde_json::Value,
    entities: &[ZepEntity],
) -> Vec<AgentConfig> {
    let agents_data = data.get("agent_configs");

    match agents_data {
        Some(arr) if arr.is_array() => {
            arr.as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, a)| AgentConfig {
                    agent_id: a.get("agent_id").and_then(|v| v.as_u64()).unwrap_or(i as u64) as usize,
                    activity_level: a.get("activity_level").and_then(|v| v.as_f64()).unwrap_or(0.5),
                    posting_probability: a.get("posting_probability").and_then(|v| v.as_f64()).unwrap_or(0.3),
                    comment_probability: a.get("comment_probability").and_then(|v| v.as_f64()).unwrap_or(0.4),
                    like_probability: a.get("like_probability").and_then(|v| v.as_f64()).unwrap_or(0.5),
                    stance: a.get("stance").and_then(|v| v.as_str()).unwrap_or("neutral").to_string(),
                    stance_strength: a.get("stance_strength").and_then(|v| v.as_f64()).unwrap_or(0.5),
                })
                .collect()
        }
        _ => {
            // Generate default configs for each entity
            entities.iter().enumerate().map(|(i, _)| AgentConfig {
                agent_id: i,
                activity_level: 0.5,
                posting_probability: 0.3,
                comment_probability: 0.4,
                like_probability: 0.5,
                stance: "neutral".to_string(),
                stance_strength: 0.5,
            }).collect()
        }
    }
}

fn parse_event_config(data: &serde_json::Value) -> EventConfig {
    let event_data = data.get("event_config");

                    let initial_posts = event_data
                        .and_then(|e| e.get("initial_posts"))
                        .and_then(|p| p.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|p| {
                                    Some(InitialPost {
                                        content: p.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                        topic: p.get("topic").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                        sentiment: p.get("sentiment").and_then(|v| v.as_str()).unwrap_or("neutral").to_string(),
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default();

    let hot_topics = event_data
        .and_then(|e| e.get("hot_topics"))
        .and_then(|p| p.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| {
                    Some(HotTopic {
                        title: t.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        description: t.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        related_entities: t.get("related_entities")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                            .unwrap_or_default(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    EventConfig {
        initial_posts,
        hot_topics,
    }
}