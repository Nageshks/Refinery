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
    timeout_secs: Option<u64>,
) -> AppResult<String> {
    let mut provider = OpenRouterProvider::new(api_key, model, endpoint);
    if let Some(t) = timeout_secs {
        provider = provider.with_timeout(t);
    }
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

#[tauri::command]
pub fn list_models(state: State<'_, AppState>) -> AppResult<Vec<crate::models::ModelConfig>> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let models = db::list_model_configs(&conn)?;
    Ok(models)
}

#[tauri::command]
pub fn save_model_config(
    state: State<'_, AppState>,
    id: String,
    provider_type: String,
    name: String,
    use_case: String,
    icon: String,
    is_custom: bool,
    enabled: Option<bool>,
) -> AppResult<crate::models::ModelConfig> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let now = Utc::now().to_rfc3339();

    // Check if it already exists to preserve created_at and enabled
    let existing = db::list_model_configs(&conn)?
        .into_iter()
        .find(|m| m.id == id);

    let existing_created_at = existing
        .as_ref()
        .map(|m| m.created_at.clone())
        .unwrap_or_else(|| now.clone());

    let enabled_val = enabled.unwrap_or_else(|| {
        existing.map(|m| m.enabled).unwrap_or(true)
    });

    let config = crate::models::ModelConfig {
        id,
        provider_type,
        name,
        use_case,
        icon,
        is_custom,
        enabled: enabled_val,
        created_at: existing_created_at,
        updated_at: now,
    };
    db::insert_model_config(&conn, &config)?;
    Ok(config)
}

#[tauri::command]
pub fn delete_model_config(state: State<'_, AppState>, model_id: String) -> AppResult<()> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::delete_model_config(&conn, &model_id)?;
    Ok(())
}

#[tauri::command]
pub fn reset_default_models(state: State<'_, AppState>) -> AppResult<Vec<crate::models::ModelConfig>> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let models = db::reset_defaults(&conn)?;
    Ok(models)
}

