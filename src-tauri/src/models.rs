use serde::{Deserialize, Serialize};

// ─── Page ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub title: String,
    pub content: String,
    pub format_type: String,
    pub created_at: String,
    pub updated_at: String,
    pub current_version_id: Option<String>,
    pub last_review_session_id: Option<String>,
    pub local_status: String,
}

// ─── ReviewSession ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSession {
    pub id: String,
    pub page_id: String,
    pub provider_id: String,
    pub model_id: String,
    pub prompt_version: String,
    pub raw_response: String,
    pub status: String,
    pub created_at: String,
}

// ─── SuggestionGroup ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionGroup {
    pub id: String,
    pub page_id: String,
    pub review_session_id: String,
    pub category: String,
    pub label: String,
    pub approval_state: String,
}

// ─── SuggestionItem ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionItem {
    pub id: String,
    pub group_id: String,
    pub original_text: String,
    pub replacement_text: String,
    pub span_start: usize,
    pub span_end: usize,
    pub explanation: String,
    pub confidence: f64,
    pub approval_state: String,
    pub conflict_state: Option<String>,
}

// ─── VersionSnapshot ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionSnapshot {
    pub id: String,
    pub page_id: String,
    pub content: String,
    pub source_review_session_id: Option<String>,
    pub applied_suggestion_ids: Vec<String>,
    pub created_at: String,
    pub name: Option<String>,
}

// ─── ProviderConfig ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub endpoint: Option<String>,
    pub selected_model: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

// ─── AI Response Parsing ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSuggestionResponse {
    pub suggestion_groups: Vec<AiSuggestionGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSuggestionGroup {
    pub category: String,
    pub label: String,
    pub items: Vec<AiSuggestionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSuggestionItem {
    pub original_text: String,
    pub replacement_text: String,
    pub explanation: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

fn default_confidence() -> f64 {
    0.5
}

// ─── Compare result ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSegment {
    pub tag: String, // "equal", "delete", "insert", "replace"
    pub old_text: String,
    pub new_text: String,
}

// ─── Review result returned to frontend ─────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub session: ReviewSession,
    pub groups: Vec<SuggestionGroupWithItems>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionGroupWithItems {
    pub group: SuggestionGroup,
    pub items: Vec<SuggestionItem>,
}

// ─── Preview result ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewResult {
    pub original_content: String,
    pub preview_content: String,
    pub highlighted_content: String,
    pub original_highlighted: String,
    pub applied_count: usize,
}

// ─── Apply result ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    pub version_id: String,
    pub applied_count: usize,
    pub new_content: String,
}
