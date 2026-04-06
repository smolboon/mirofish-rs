//! LLM response parsing utilities

use serde::de::DeserializeOwned;

/// Parse a JSON response from the LLM
pub fn parse_json_response<T: DeserializeOwned>(content: &str) -> Result<T, String> {
    // Try to find JSON in the response (LLMs may include markdown code blocks)
    let json_str = extract_json_block(content);
    serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse JSON: {}", e))
}

/// Extract JSON from markdown code blocks if present
pub fn extract_json_block(content: &str) -> String {
    // Try to find ```json ... ``` block
    if let Some(start) = content.find("```json") {
        let rest = &content[start + 7..];
        if let Some(end) = rest.find("```") {
            return rest[..end].trim().to_string();
        }
    }
    // Try to find ``` ... ``` block (without language tag)
    if let Some(start) = content.find("```") {
        let rest = &content[start + 3..];
        if let Some(end) = rest.find("```") {
            return rest[..end].trim().to_string();
        }
    }
    // Return content as-is
    content.trim().to_string()
}

/// Parse ReACT Thought from response
pub fn parse_react_thought(content: &str) -> Option<String> {
    if let Some(start) = content.find("Thought:") {
        let rest = &content[start + 8..];
        let end = rest.find('\n').unwrap_or(rest.len());
        Some(rest[..end].trim().to_string())
    } else {
        None
    }
}

/// Parse ReACT Action (tool call) from response
pub fn parse_react_action(content: &str) -> Option<(String, String)> {
    // Look for <tool_call>{"name": ..., "parameters": ...}</tool_call>
    if let Some(start) = content.find("<tool_call>") {
        let rest = &content[start + 3..];
        if let Some(end) = rest.find("</tool_call>") {
            let tool_json = &rest[..end].trim();
            // Try to parse as JSON to extract name and parameters
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(tool_json) {
                let name = value.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                return Some((name, tool_json.to_string()));
            }
        }
    }
    None
}

/// Parse ReACT Final Answer from response
pub fn parse_final_answer(content: &str) -> Option<String> {
    if let Some(start) = content.find("Final Answer:") {
        Some(content[start + 13..].trim().to_string())
    } else {
        None
    }
}