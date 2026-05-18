mod app;
mod commands;
mod domain;
mod infrastructure;
mod utils;

use commands::import_commands::{
    get_import_batch_detail, import_monthly_report_csv, list_import_batches,
    preview_monthly_report_csv, search_import_batches,
};
use commands::system_commands::get_system_info;
use commands::xlsx_markdown_commands::convert_xlsx_spec_to_markdown;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_system_info,
            preview_monthly_report_csv,
            import_monthly_report_csv,
            list_import_batches,
            search_import_batches,
            get_import_batch_detail,
            convert_xlsx_spec_to_markdown
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
