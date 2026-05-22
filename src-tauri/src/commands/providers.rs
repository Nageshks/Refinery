use tauri::State;
use uuid::Uuid;
use chrono::Utc;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::ProviderConfig;
use crate::providers::openrouter::OpenRouterProvider;
use crate::providers::AiProvider;
use crate::AppState;

#[tauri::command]
pub fn list_providers(state: State<'_, AppState>) -> AppResult<Vec<ProviderConfig>> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let configs = db::list_provider_configs(&conn)?;
    Ok(configs)
}

#[tauri::command]
pub fn get_active_provider(state: State<'_, AppState>) -> AppResult<Option<ProviderConfig>> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let configs = db::list_provider_configs(&conn)?;
    Ok(configs.into_iter().find(|c| c.enabled))
}

#[tauri::command]
pub fn set_active_provider(state: State<'_, AppState>, provider_id: String) -> AppResult<()> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    // Disable all, then enable the selected one
    let configs = db::list_provider_configs(&conn)?;
    for config in &configs {
        let mut updated = config.clone();
        updated.enabled = config.id == provider_id;
        updated.updated_at = Utc::now().to_rfc3339();
        db::insert_provider_config(&conn, &updated)?;
    }
    Ok(())
}

#[tauri::command]
pub fn save_provider_config(
    state: State<'_, AppState>,
    id: Option<String>,
    name: String,
    provider_type: String,
    endpoint: Option<String>,
    selected_model: String,
    enabled: bool,
) -> AppResult<ProviderConfig> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let now = Utc::now().to_rfc3339();
    let config = ProviderConfig {
        id: id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        name,
        provider_type,
        endpoint,
        selected_model,
        enabled,
        created_at: now.clone(),
        updated_at: now,
    };
    db::insert_provider_config(&conn, &config)?;
    Ok(config)
}

#[tauri::command]
pub fn delete_provider_config(state: State<'_, AppState>, provider_id: String) -> AppResult<()> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::delete_provider_config(&conn, &provider_id)?;
    Ok(())
}

#[tauri::command]
pub async fn test_provider(
    api_key: String,
    model: String,
    endpoint: Option<String>,
) -> AppResult<String> {
    let provider = OpenRouterProvider::new(api_key, model, endpoint);
    let result = provider
        .review(
            "Hello world.",
            "Return exactly: {\"suggestion_groups\": []}",
            "Test connection. Return valid JSON.",
        )
        .await
        .map_err(|e| AppError::Provider(e.to_string()))?;
    Ok(format!("Connection successful. Response length: {} chars", result.len()))
}
