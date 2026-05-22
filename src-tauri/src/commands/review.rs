use tauri::State;
use uuid::Uuid;
use chrono::Utc;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::engine::{suggestion, patch, prompts};
use crate::providers::openrouter::OpenRouterProvider;
use crate::providers::AiProvider;
use crate::AppState;

#[tauri::command]
pub async fn start_review(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    page_id: String,
    api_key: String,
    model: String,
    endpoint: Option<String>,
) -> AppResult<ReviewResult> {
    // 1. Load page content
    let (page_content, session_id, provider_id) = {
        let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        let page = db::get_page(&conn, &page_id)?
            .ok_or_else(|| AppError::NotFound(format!("Page '{}' not found", page_id)))?;

        if page.content.trim().is_empty() {
            return Err(AppError::Validation("Page content is empty. Write something first.".to_string()));
        }

        let provider_id = db::list_provider_configs(&conn)?
            .into_iter()
            .find(|c| c.enabled)
            .map(|c| c.provider_type)
            .unwrap_or_else(|| "openrouter".to_string());

        // 2. Create review session record
        let session_id = Uuid::new_v4().to_string();
        let session = ReviewSession {
            id: session_id.clone(),
            page_id: page_id.clone(),
            provider_id: provider_id.clone(),
            model_id: model.clone(),
            prompt_version: prompts::PROMPT_VERSION.to_string(),
            raw_response: String::new(),
            status: "pending".to_string(),
            created_at: Utc::now().to_rfc3339(),
        };
        db::insert_review_session(&conn, &session)?;
        db::set_page_review_session(&conn, &page_id, &session_id)?;

        (page.content, session_id, provider_id)
    };

    // 3. Call AI provider (outside lock)
    let provider = OpenRouterProvider::new(api_key, model, endpoint);
    let system_prompt = prompts::review_system_prompt();
    let user_prompt = prompts::review_user_prompt(&page_content);

    let raw_response = provider
        .review(&page_content, system_prompt, &user_prompt)
        .await
        .map_err(|e| AppError::Provider(e.to_string()))?;

    // 4. Normalize response into structured suggestions
    let groups_with_items = suggestion::normalize_ai_response(
        &raw_response, &page_content, &page_id, &session_id,
    ).map_err(|e| AppError::Provider(format!("Failed to normalize AI response: {}", e)))?;

    // 5. Store everything in DB
    {
        let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
        db::update_review_session_response(&conn, &session_id, &raw_response, "completed")?;

        for gwi in &groups_with_items {
            db::insert_suggestion_group(&conn, &gwi.group)?;
            for item in &gwi.items {
                db::insert_suggestion_item(&conn, item)?;
            }
        }

        let session = ReviewSession {
            id: session_id.clone(),
            page_id: page_id.clone(),
            provider_id,
            model_id: String::new(),
            prompt_version: prompts::PROMPT_VERSION.to_string(),
            raw_response: raw_response.clone(),
            status: "completed".to_string(),
            created_at: Utc::now().to_rfc3339(),
        };

        Ok(ReviewResult {
            session,
            groups: groups_with_items,
        })
    }
}

#[tauri::command]
pub fn get_review_suggestions(
    state: State<'_, AppState>,
    session_id: String,
) -> AppResult<Vec<SuggestionGroupWithItems>> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let groups = db::get_suggestion_groups_with_items(&conn, &session_id)?;
    Ok(groups)
}

#[tauri::command]
pub fn update_suggestion_approval(
    state: State<'_, AppState>,
    item_id: Option<String>,
    group_id: Option<String>,
    approval_state: String,
) -> AppResult<()> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;

    if let Some(gid) = group_id {
        db::update_group_approval(&conn, &gid, &approval_state)?;
    } else if let Some(iid) = item_id {
        db::update_item_approval(&conn, &iid, &approval_state)?;
    } else {
        return Err(AppError::Validation("Must provide either item_id or group_id".to_string()));
    }

    Ok(())
}

#[tauri::command]
pub fn compute_preview(
    state: State<'_, AppState>,
    page_id: String,
    session_id: String,
) -> AppResult<PreviewResult> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;

    let page = db::get_page(&conn, &page_id)?
        .ok_or_else(|| AppError::NotFound(format!("Page '{}' not found", page_id)))?;

    let approved_items = db::get_approved_items_for_session(&conn, &session_id)?;
    let all_items = db::get_all_items_for_session(&conn, &session_id)?;

    // Filter out rejected suggestions and resolve overlaps/conflicts so we don't crash apply_patches_highlighted
    let non_rejected_items: Vec<SuggestionItem> = all_items.iter()
        .filter(|item| item.approval_state != "rejected")
        .cloned()
        .collect();

    let mut proposed_highlight_items: Vec<SuggestionItem> = Vec::new();
    for item in non_rejected_items {
        let mut conflicts = false;
        for existing in &proposed_highlight_items {
            if item.span_start < existing.span_end && existing.span_start < item.span_end {
                conflicts = true;
                break;
            }
        }
        if !conflicts {
            proposed_highlight_items.push(item);
        }
    }

    let preview_content = patch::compute_preview(&page.content, &approved_items)
        .map_err(|e| AppError::Patch(e))?;

    // Now highlighted_content will contain all non-rejected, conflict-free items applied and wrapped in spans
    let highlighted_content = patch::apply_patches_highlighted(&page.content, &proposed_highlight_items)
        .map_err(|e| AppError::Patch(e))?;

    // Filter out overlapping items for original highlights as well, keeping the first suggestion in case of duplicates/alternatives to prevent index shift / mismatch errors
    let mut original_highlight_items: Vec<SuggestionItem> = Vec::new();
    for item in &all_items {
        let mut conflicts = false;
        for existing in &original_highlight_items {
            if item.span_start < existing.span_end && existing.span_start < item.span_end {
                conflicts = true;
                break;
            }
        }
        if !conflicts {
            original_highlight_items.push(item.clone());
        }
    }

    let original_highlighted = patch::highlight_original(&page.content, &original_highlight_items)
        .map_err(|e| AppError::Patch(e))?;

    Ok(PreviewResult {
        original_content: page.content,
        preview_content,
        highlighted_content,
        original_highlighted,
        applied_count: approved_items.len(),
    })
}

#[tauri::command]
pub fn apply_approved_suggestions(
    state: State<'_, AppState>,
    page_id: String,
    session_id: String,
) -> AppResult<ApplyResult> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;

    let page = db::get_page(&conn, &page_id)?
        .ok_or_else(|| AppError::NotFound(format!("Page '{}' not found", page_id)))?;

    let approved_items = db::get_approved_items_for_session(&conn, &session_id)?;

    if approved_items.is_empty() {
        return Err(AppError::Validation("No approved suggestions to apply".to_string()));
    }

    // Apply patches
    let new_content = patch::apply_patches(&page.content, &approved_items)
        .map_err(|e| AppError::Patch(e))?;

    // Create version snapshot
    let version_id = Uuid::new_v4().to_string();
    let applied_ids: Vec<String> = approved_items.iter().map(|i| i.id.clone()).collect();
    let snapshot = VersionSnapshot {
        id: version_id.clone(),
        page_id: page_id.clone(),
        content: page.content.clone(), // Store the pre-apply content
        source_review_session_id: Some(session_id.clone()),
        applied_suggestion_ids: applied_ids,
        created_at: Utc::now().to_rfc3339(),
        name: None,
    };
    db::insert_version_snapshot(&conn, &snapshot)?;

    // Update page with new content
    db::update_page_content(&conn, &page_id, &page.title, &new_content)?;
    db::set_page_version(&conn, &page_id, &version_id)?;

    Ok(ApplyResult {
        version_id,
        applied_count: approved_items.len(),
        new_content,
    })
}

#[tauri::command]
pub async fn rewrite_selection(
    api_key: String,
    model: String,
    endpoint: Option<String>,
    selected_text: String,
) -> AppResult<Vec<String>> {
    let provider = OpenRouterProvider::new(api_key, model, endpoint);
    let system_prompt = "You are an expert editing assistant. Rewrite the user's selected text in exactly 3 distinct, high-quality styles (e.g. professional/clear, fluent/natural, bold/engaging). Return ONLY a JSON array of strings containing the 3 rewrites (e.g. [\"Option 1\", \"Option 2\", \"Option 3\"]). DO NOT include markdown code blocks, explanation text, or extra characters.";
    let user_prompt = format!("Rewrite the following selection:\n\n\"{}\"", selected_text);

    let raw_response = provider
        .compare(system_prompt, &user_prompt)
        .await
        .map_err(|e| AppError::Provider(e.to_string()))?;

    let clean = raw_response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let rewrites: Vec<String> = serde_json::from_str(clean).unwrap_or_else(|_| {
        // Fallback line parsing
        raw_response
            .lines()
            .map(|l| {
                l.trim()
                    .trim_start_matches('-')
                    .trim_start_matches(|c: char| c.is_ascii_digit())
                    .trim_start_matches('.')
                    .trim()
                    .replace('"', "")
            })
            .filter(|l| !l.is_empty() && l.len() > 2)
            .take(3)
            .collect()
    });

    Ok(rewrites)
}

#[tauri::command]
pub fn clear_review_session(
    state: State<'_, AppState>,
    page_id: String,
) -> AppResult<()> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::clear_page_review_session(&conn, &page_id)?;
    Ok(())
}
