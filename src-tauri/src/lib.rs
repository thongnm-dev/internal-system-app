//! Entry point của ứng dụng Tauri.
//!
//! Khởi tạo Tauri Builder, đăng ký plugin, thiết lập database khi khởi động,
//! và đăng ký tất cả các Tauri command handlers cho frontend gọi qua IPC.

// Khai báo cấu trúc module tree — mỗi tầng kiến trúc (app/commands/database/models/
// services/utils) có file khai báo riêng dưới `modules/`, `include!()` vào crate root
// để tên module giữ nguyên (`crate::services::...`) như thể khai báo trực tiếp tại đây.
include!("modules/app.rs");
include!("modules/commands.rs");
include!("modules/database.rs");
include!("modules/models.rs");
include!("modules/services.rs");
include!("modules/utils.rs");

use tauri::Manager;

// Danh sách toàn bộ Tauri command handlers, tách riêng cho gọn `lib.rs`. Dùng `include!()`
// (không phải `mod invoke_handler;`) để nội dung được ghép trực tiếp vào crate root — các
// macro `__cmd__*` ẩn mà `#[tauri::command]` sinh ra chỉ được export ở crate root nên cần
// giữ nguyên ngữ cảnh này, không được nằm trong submodule riêng.
include!("invoke_handler.rs");

/// Khởi chạy ứng dụng Tauri desktop.
///
/// Thứ tự khởi tạo:
/// 1. Đăng ký plugin dialog (cho file picker, message box, v.v.)
/// 2. Setup hook: khởi tạo database (tạo bảng + stored procedure) chạy nền
/// 3. Đăng ký toàn bộ IPC command handlers
/// 4. Chạy event loop của ứng dụng
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            utils::logger::init();

            if let Some(window) = app.get_webview_window("main") {
                let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/icon.ico"))
                    .expect("failed to load app icon");
                let _ = window.set_icon(icon);

                // Trên macOS cờ `maximized` trong tauri.conf.json không áp dụng ổn định,
                // nên chủ động maximize cửa sổ khi khởi động.
                let _ = window.maximize();
            }

            // Khởi tạo database chạy nền (chỉ dev — production dùng bảng/SP có sẵn)
            if cfg!(debug_assertions) {
                tauri::async_runtime::spawn(async {
                    if let Err(e) = database::startup_store::init().await {
                        log::error!("Failed to initialize database tables: {e}");
                    }
                });
            }

            // Nạp trước dữ liệu AI Usage và chạy poll nền để theo dõi usage + auto-switch.
            services::ai_acc_service::preload();
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                services::ai_usage_service::run_poll_loop(handle).await;
            });

            // Theo dõi nền storage S3 → bắn notification khi có tài liệu mới.
            let s3_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                services::s3_watch_service::run_poll_loop(s3_handle).await;
            });
            Ok(())
        })
        .invoke_handler(build_invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
