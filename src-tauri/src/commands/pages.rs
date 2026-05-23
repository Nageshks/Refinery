use tauri::State;
use uuid::Uuid;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::Page;
use crate::AppState;

#[tauri::command]
pub fn list_pages(state: State<'_, AppState>) -> AppResult<Vec<Page>> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let pages = db::list_pages(&conn)?;
    Ok(pages)
}

#[tauri::command]
pub fn get_page(state: State<'_, AppState>, page_id: String) -> AppResult<Page> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::get_page(&conn, &page_id)?
        .ok_or_else(|| AppError::NotFound(format!("Page '{}' not found", page_id)))
}

#[tauri::command]
pub fn create_page(state: State<'_, AppState>, title: String) -> AppResult<Page> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    let id = Uuid::new_v4().to_string();
    db::insert_page(&conn, &id, &title, "")?;
    db::get_page(&conn, &id)?
        .ok_or_else(|| AppError::NotFound("Failed to retrieve created page".to_string()))
}

#[tauri::command]
pub fn update_page(
    state: State<'_, AppState>,
    page_id: String,
    title: String,
    content: String,
) -> AppResult<Page> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::update_page_content(&conn, &page_id, &title, &content)?;
    db::get_page(&conn, &page_id)?
        .ok_or_else(|| AppError::NotFound(format!("Page '{}' not found", page_id)))
}

#[tauri::command]
pub fn rename_page(state: State<'_, AppState>, page_id: String, title: String) -> AppResult<Page> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::rename_page(&conn, &page_id, &title)?;
    db::get_page(&conn, &page_id)?
        .ok_or_else(|| AppError::NotFound(format!("Page '{}' not found", page_id)))
}

#[tauri::command]
pub fn delete_page(state: State<'_, AppState>, page_id: String) -> AppResult<()> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::delete_page(&conn, &page_id)?;
    Ok(())
}

#[tauri::command]
pub fn update_page_format(state: State<'_, AppState>, page_id: String, format_type: String) -> AppResult<Page> {
    let conn = state.db.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    db::update_page_format(&conn, &page_id, &format_type)?;
    db::get_page(&conn, &page_id)?
        .ok_or_else(|| AppError::NotFound(format!("Page '{}' not found", page_id)))
}
