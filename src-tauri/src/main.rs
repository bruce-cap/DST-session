// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod index;
mod json_util;
mod launcher;
mod model;
mod paths;
mod providers;
mod shell;
mod state;
mod time;
mod usage;

fn main() {
    tauri::Builder::default()
        .manage(providers::ProviderRegistry::bootstrap())
        .invoke_handler(tauri::generate_handler![
            commands::list_providers,
            commands::list_sessions,
            commands::refresh_sessions,
            commands::get_source_state,
            commands::get_token_usage,
            commands::refresh_token_usage,
            commands::get_app_state,
            commands::set_favorite,
            commands::set_deepseek_launcher,
            commands::set_provider_launcher,
            commands::set_auto_refresh,
            commands::check_agent,
            commands::open_session_folder,
            commands::resume_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
