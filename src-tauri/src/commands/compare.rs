use serde::{Deserialize, Serialize};
use crate::error::{AppError, AppResult};
use crate::engine::compare::generate_diff;
use crate::engine::prompts;
use crate::models::DiffSegment;
use crate::providers::openrouter::OpenRouterProvider;
use crate::providers::AiProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareResult {
    pub diff_segments: Vec<DiffSegment>,
    pub ai_analysis: Option<String>,
}

#[tauri::command]
pub async fn compare_texts(
    text_a: String,
    text_b: String,
    api_key: Option<String>,
    model: Option<String>,
    endpoint: Option<String>,
) -> AppResult<CompareResult> {
    // Always generate local diff
    let diff_segments = generate_diff(&text_a, &text_b);

    // Optionally run AI comparison analysis
    let ai_analysis = if let (Some(key), Some(mdl)) = (api_key, model) {
        let provider = OpenRouterProvider::new(key, mdl, endpoint);
        let system_prompt = prompts::compare_system_prompt();
        let user_prompt = prompts::compare_user_prompt(&text_a, &text_b);
        match provider.compare(system_prompt, &user_prompt).await {
            Ok(response) => Some(response),
            Err(e) => {
                log::warn!("AI comparison failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    Ok(CompareResult {
        diff_segments,
        ai_analysis,
    })
}
