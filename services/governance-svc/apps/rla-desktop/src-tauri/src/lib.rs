//! src-tauri crate.
#![warn(missing_docs)]
pub mod commands;
pub mod db;
pub mod error;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir().expect("data dir");
            std::fs::create_dir_all(&data_dir).ok();
            let db_path = format!("sqlite:{}/rla.db", data_dir.display());
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            let pool = rt.block_on(db::init_db(&db_path)).expect("db init");
            app.manage(pool);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_ballots,
            commands::record_verdict,
            commands::pending_sync_count,
            commands::mark_synced,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
