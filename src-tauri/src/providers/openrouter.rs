use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use super::{AiProvider, ProviderError};

pub struct OpenRouterProvider {
    client: Client,
    api_key: String,
    model: String,
    endpoint: String,
}

impl OpenRouterProvider {
    pub fn new(api_key: String, model: String, endpoint: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
            api_key,
            model,
            endpoint: endpoint.unwrap_or_else(|| "https://openrouter.ai/api/v1/chat/completions".to_string()),
        }
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Option<Vec<ChatChoice>>,
    error: Option<ChatError>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct ChatError {
    message: String,
    #[serde(default)]
    code: Option<i32>,
}

#[async_trait]
impl AiProvider for OpenRouterProvider {
    async fn review(&self, _content: &str, system_prompt: &str, user_prompt: &str) -> Result<String, ProviderError> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage { role: "system".to_string(), content: system_prompt.to_string() },
                ChatMessage { role: "user".to_string(), content: user_prompt.to_string() },
            ],
            temperature: 0.3,
            max_tokens: 4096,
        };

        let response = self.client
            .post(&self.endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://refinery.desktop")
            .header("X-Title", "Refinery")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(ProviderError::Auth(format!("Authentication failed ({}): {}", status, body)));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ProviderError::RateLimit("Rate limit exceeded. Please wait and try again.".to_string()));
        }
        if !status.is_success() {
            return Err(ProviderError::Unknown(format!("HTTP {}: {}", status, body)));
        }

        let parsed: ChatResponse = serde_json::from_str(&body)
            .map_err(|e| ProviderError::MalformedOutput(format!("Failed to parse response: {} — body: {}", e, &body[..body.len().min(500)])))?;

        if let Some(err) = parsed.error {
            return Err(ProviderError::Unknown(err.message));
        }

        let content = parsed.choices
            .and_then(|c| c.into_iter().next())
            .map(|c| c.message.content)
            .ok_or_else(|| ProviderError::MalformedOutput("No choices in response".to_string()))?;

        Ok(content)
    }

    async fn compare(&self, system_prompt: &str, user_prompt: &str) -> Result<String, ProviderError> {
        // Same endpoint, just different prompts
        self.review("", system_prompt, user_prompt).await
    }

    fn provider_name(&self) -> &str {
        "openrouter"
    }
}
