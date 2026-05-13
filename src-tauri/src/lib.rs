mod app;
mod commands;
mod domain;
mod infrastructure;
mod utils;

use commands::statistics_commands::analyze_csv;
use commands::system_commands::get_system_info;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![analyze_csv, get_system_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
