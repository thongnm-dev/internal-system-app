//! Các service con của công cụ đồng bộ tài liệu VN → JP.
//!
//! `sync_service` chứa toàn bộ xử lý CHUNG (không phụ thuộc loại tài liệu); mỗi `cXXX_sync_service`
//! chỉ chứa phần xử lý RIÊNG của loại tài liệu đó (vùng cột nội dung, và với C2.3.8 — canh dòng
//! theo group). API công khai cho Tauri command nằm ở `super::vnjp_sync_service`.

pub mod sync_service;

/// C2.3.2 プログラム処理概要図.
pub mod c232_sync_service;
/// C2.3.3 イベント詳細設計書.
pub mod c233_sync_service;
/// C2.3.4 画面仕様書（編集要領）.
pub mod c234_sync_service;
/// C2.3.5 画面仕様書（単独チェック）.
pub mod c235_sync_service;
/// C2.3.6 画面仕様書（相関チェック）.
pub mod c236_sync_service;
/// C2.3.8 画面間インタフェース仕様書.
pub mod c238_sync_service;
