use tauri::State;
use uuid::Uuid;
use chrono::Utc;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::VersionSnapshot;
use crate::AppState;

#[tauri::command]
pub fn list_versions(state: State<'_, AppState>, page_id: String) -> AppResult<Vec<VersionSnapshot>> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let versions = db::list_versions(&conn, &page_id)?;
    Ok(versions)
}

#[tauri::command]
pub fn get_version(state: State<'_, AppState>, version_id: String) -> AppResult<VersionSnapshot> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::get_version(&conn, &version_id)?
        .ok_or_else(|| AppError::NotFound(format!("Version '{}' not found", version_id)))
}

#[tauri::command]
pub fn restore_version(
    state: State<'_, AppState>,
    page_id: String,
    version_id: String,
) -> AppResult<String> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;

    let version = db::get_version(&conn, &version_id)?
        .ok_or_else(|| AppError::NotFound(format!("Version '{}' not found", version_id)))?;

    let page = db::get_page(&conn, &page_id)?
        .ok_or_else(|| AppError::NotFound(format!("Page '{}' not found", page_id)))?;

    // Restore the page content from the version snapshot
    db::update_page_content(&conn, &page_id, &page.title, &version.content)?;

    Ok(version.content)
}

#[tauri::command]
pub fn rename_version(
    state: State<'_, AppState>,
    version_id: String,
    name: String,
) -> AppResult<()> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::rename_version(&conn, &version_id, &name)?;
    Ok(())
}

#[tauri::command]
pub fn create_manual_version(
    state: State<'_, AppState>,
    page_id: String,
    name: String,
) -> AppResult<VersionSnapshot> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    
    let page = db::get_page(&conn, &page_id)?
        .ok_or_else(|| AppError::NotFound(format!("Page '{}' not found", page_id)))?;

    let version_id = Uuid::new_v4().to_string();
    let snapshot = VersionSnapshot {
        id: version_id.clone(),
        page_id: page_id.clone(),
        content: page.content.clone(),
        source_review_session_id: None,
        applied_suggestion_ids: Vec::new(),
        created_at: Utc::now().to_rfc3339(),
        name: Some(name),
    };

    db::insert_version_snapshot(&conn, &snapshot)?;
    db::set_page_version(&conn, &page_id, &version_id)?;

    Ok(snapshot)
}
