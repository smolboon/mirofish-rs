//! LLM Client using async-openai (OpenAI compatible API)

use std::sync::Arc;

use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequest,
        CreateChatCompletionRequestArgs,
        FinishReason,
        ResponseFormat,
    },
    Client,
};
use serde::de::DeserializeOwned;
use tracing::{debug, instrument};

use mirofish_core::{AppConfig, LlmError};

/// LLM Client for making API calls
#[derive(Clone)]
pub struct LLMClient {
    client: Arc<Client<OpenAIConfig>>,
    model: String,
    max_tokens: Option<u32>,
    temperature: f32,
}

// Ensure LLMClient is Send + Sync + 'static for axum State
static_assertions::assert_impl_all!(LLMClient: Send, Sync);

impl LLMClient {
    /// Create a new LLM client from configuration
    pub fn new(config: &AppConfig) -> Self {
        let openai_config = OpenAIConfig::new()
            .with_api_key(&config.llm_api_key)
            .with_api_base(&config.llm_base_url);

        Self {
            client: Arc::new(Client::with_config(openai_config)),
            model: config.llm_model_name.clone(),
            max_tokens: None,
            temperature: 0.7,
        }
    }

    /// Create with custom temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Simple chat completion - returns text response
    #[instrument(skip(self, system_prompt, user_prompt), fields(model = %self.model))]
    pub async fn chat(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> std::result::Result<String, LlmError> {
        debug!("Chat request: system={}, user={}", system_prompt, user_prompt);

        let messages = self.build_messages(system_prompt, user_prompt, None);
        let request = self.build_request(messages, false)?;

        let response = self.client
            .chat()
            .create(request)
            .await
            .map_err(|e| LlmError::Api(e.to_string()))?;

        let content = extract_choice_content(&response)?;
        debug!("Chat response: {} chars", content.len());

        Ok(content)
    }

    /// JSON-structured chat completion - parses response as JSON
    #[instrument(skip(self, system_prompt, user_prompt), fields(model = %self.model))]
    pub async fn chat_json<T: DeserializeOwned>(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> std::result::Result<T, LlmError> {
        debug!("JSON chat request: system={}, user={}", system_prompt, user_prompt);

        let messages = self.build_messages(system_prompt, user_prompt, None);
        let request = self.build_request(messages, true)?;

        let response = self.client
            .chat()
            .create(request)
            .await
            .map_err(|e| LlmError::Api(e.to_string()))?;

        let content = extract_choice_content(&response)?;
        debug!("JSON chat response: {}", content);

        serde_json::from_str(&content)
            .map_err(|e| LlmError::ParseError(format!("Failed to parse JSON: {}", e)))
    }

    /// Chat with conversation history
    #[instrument(skip(self, messages))]
    pub async fn chat_with_history(
        &self,
        system_prompt: &str,
        messages: Vec<ChatCompletionRequestMessage>,
    ) -> std::result::Result<String, LlmError> {
        let mut all_messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()
                .map_err(|e| LlmError::Api(format!("Failed to build system message: {}", e)))?
                .into(),
        ];
        all_messages.extend(messages);

        let request = self.build_request(all_messages, false)?;

        let response = self.client
            .chat()
            .create(request)
            .await
            .map_err(|e| LlmError::Api(e.to_string()))?;

        extract_choice_content(&response)
    }

    fn build_messages(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        _history: Option<Vec<ChatCompletionRequestMessage>>,
    ) -> Vec<ChatCompletionRequestMessage> {
        let mut messages: Vec<ChatCompletionRequestMessage> = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()
                .unwrap()
                .into(),
        ];

        messages.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_prompt)
                .build()
                .unwrap()
                .into(),
        );

        messages
    }

    fn build_request(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
        json_mode: bool,
    ) -> std::result::Result<CreateChatCompletionRequest, LlmError> {
        let mut builder = CreateChatCompletionRequestArgs::default();

        builder
            .model(&self.model)
            .temperature(self.temperature)
            .messages(messages);

        if json_mode {
            builder.response_format(ResponseFormat::JsonObject);
        }

        if let Some(max_tokens) = self.max_tokens {
            builder.max_tokens(max_tokens);
        }

        builder.build().map_err(|e| LlmError::Api(format!("Failed to build request: {}", e)))
    }
}

/// Extract content from the first choice in the response
fn extract_choice_content(
    response: &async_openai::types::CreateChatCompletionResponse,
) -> std::result::Result<String, LlmError> {
    let choice = response.choices.first().ok_or_else(|| {
        LlmError::Api("No choices in response".to_string())
    })?;

    if let Some(finish_reason) = &choice.finish_reason {
        if matches!(finish_reason, FinishReason::Length) {
            return Err(LlmError::Api("Response was truncated due to max tokens".to_string()));
        }
    }

    match &choice.message.content {
        Some(content) => Ok(content.clone()),
        None => Err(LlmError::Api("No content in response".to_string())),
    }
}

/// Extract tool calls from response
pub fn extract_tool_calls(
    response: &async_openai::types::CreateChatCompletionResponse,
) -> Option<Vec<async_openai::types::ChatCompletionMessageToolCall>> {
    response.choices.first()?.message.tool_calls.clone()
}