//! src-tauri crate.
#![warn(missing_docs)]
pub mod commands;
pub mod db;
pub mod error;

use commands::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let db_path = app
                .path()
                .app_data_dir()
                .expect("app data dir missing")
                .join("ro-desktop.db");

            let db_path_str = db_path.to_str().expect("db path not utf-8").to_string();

            tauri::async_runtime::block_on(async move {
                let pool = db::open_db(&db_path_str)
                    .await
                    .expect("failed to open local SQLite database");
                app.manage(AppState {
                    db: pool,
                    backend_url: std::env::var("RTS_RO_API_URL")
                        .unwrap_or_else(|_| "http://localhost:8083".to_string()),
                });
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_queue,
            commands::review_form34a,
            commands::compile_34b,
            commands::create_dispute,
            commands::session_recording,
            commands::get_pending_actions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ro-desktop");
}
