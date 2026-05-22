pub mod openrouter;

use async_trait::async_trait;

#[derive(Debug)]
pub enum ProviderError {
    Network(String),
    Auth(String),
    RateLimit(String),
    MalformedOutput(String),
    Timeout(String),
    Unknown(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::Network(e) => write!(f, "Network error: {}", e),
            ProviderError::Auth(e) => write!(f, "Authentication error: {}", e),
            ProviderError::RateLimit(e) => write!(f, "Rate limit: {}", e),
            ProviderError::MalformedOutput(e) => write!(f, "Malformed output: {}", e),
            ProviderError::Timeout(e) => write!(f, "Timeout: {}", e),
            ProviderError::Unknown(e) => write!(f, "Unknown error: {}", e),
        }
    }
}

impl From<reqwest::Error> for ProviderError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            ProviderError::Timeout(e.to_string())
        } else if e.is_connect() {
            ProviderError::Network(e.to_string())
        } else {
            ProviderError::Unknown(e.to_string())
        }
    }
}

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn review(&self, content: &str, system_prompt: &str, user_prompt: &str) -> Result<String, ProviderError>;
    async fn compare(&self, system_prompt: &str, user_prompt: &str) -> Result<String, ProviderError>;
    fn provider_name(&self) -> &str;
}
