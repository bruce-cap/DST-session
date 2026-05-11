mod commands;
mod json_util;
mod launcher;
mod model;
mod paths;
mod providers;
mod shell;
mod state;
mod time;

fn main() {
    tauri::Builder::default()
        .manage(providers::ProviderRegistry::bootstrap())
        .invoke_handler(tauri::generate_handler![
            commands::list_providers,
            commands::list_sessions,
            commands::get_app_state,
            commands::set_favorite,
            commands::set_deepseek_launcher,
            commands::check_agent,
            commands::open_session_folder,
            commands::resume_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
