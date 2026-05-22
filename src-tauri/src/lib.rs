mod error;
mod models;
mod db;
mod commands;
mod providers;
mod engine;

use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir).ok();
            let db_path = app_data_dir.join("refinery.db");

            let conn = db::init_db(&db_path).expect("failed to initialize database");
            app.manage(AppState { db: Mutex::new(conn) });

            log::info!("Refinery initialized. DB at: {:?}", db_path);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Pages
            commands::pages::list_pages,
            commands::pages::get_page,
            commands::pages::create_page,
            commands::pages::update_page,
            commands::pages::rename_page,
            commands::pages::delete_page,
            // Review
            commands::review::start_review,
            commands::review::get_review_suggestions,
            commands::review::update_suggestion_approval,
            commands::review::compute_preview,
            commands::review::apply_approved_suggestions,
            commands::review::rewrite_selection,
            commands::review::clear_review_session,
            // Providers
            commands::providers::list_providers,
            commands::providers::get_active_provider,
            commands::providers::set_active_provider,
            commands::providers::save_provider_config,
            commands::providers::delete_provider_config,
            commands::providers::test_provider,
            // Compare
            commands::compare::compare_texts,
            // Versions
            commands::versions::list_versions,
            commands::versions::get_version,
            commands::versions::restore_version,
            commands::versions::rename_version,
            commands::versions::create_manual_version,
            // Window controls (custom Rust fallback)
            commands::window::minimize_window,
            commands::window::maximize_window,
            commands::window::close_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
