//! Điểm khởi chạy (entry point) của ứng dụng desktop.
//!
//! Gọi `app::run()` để khởi tạo Tauri runtime và mở cửa sổ ứng dụng.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    app::run()
}
