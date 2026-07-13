//! Service lấy thông tin hệ thống.

use crate::models::system::SystemInfo;
use crate::utils::network::local_ip_address;
use crate::utils::time::current_timestamp;
use std::env;

/// Thu thập thông tin hệ thống hiện tại: username, timestamp, IP, phiên bản app.
pub fn get_system_info() -> SystemInfo {
    SystemInfo {
        username: env::var("USERNAME")
            .or_else(|_| env::var("USER"))
            .unwrap_or_else(|_| "unknown".to_string()),
        timestamp: current_timestamp(),
        ip_address: local_ip_address(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}
