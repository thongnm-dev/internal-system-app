//! Model cho module phân quyền menu (governance).

use serde::{Deserialize, Serialize};

/// Quyền menu của một user sau khi đã gộp role + override riêng.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EffectiveMenuPermission {
    pub menu_key: String,
    pub is_allowed: bool,
    /// Quyền suy ra từ các role của user, trước khi áp override riêng.
    pub role_allowed: bool,
    /// `user` nếu do override riêng quyết định, `role` nếu suy ra từ role.
    pub source: String,
}

/// Override quyền menu ở cấp user.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserMenuPermission {
    pub menu_key: String,
    pub is_allowed: bool,
}

/// Request lưu danh sách menu được cấp cho một role.
#[derive(Debug, Deserialize)]
pub struct SaveRoleMenuPermissionsRequest {
    pub role_id: i32,
    pub menu_keys: Vec<String>,
}

/// Request lưu override quyền menu của một user.
///
/// Menu không nằm trong `allow_keys` lẫn `deny_keys` sẽ theo quyền của role.
#[derive(Debug, Deserialize)]
pub struct SaveUserMenuPermissionsRequest {
    pub user_id: i32,
    pub allow_keys: Vec<String>,
    pub deny_keys: Vec<String>,
}
