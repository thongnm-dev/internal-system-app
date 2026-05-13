use serde::Serialize;

#[derive(Serialize)]
pub struct SystemInfo {
    pub username: String,
    pub timestamp: String,
    pub ip_address: String,
    pub version: String,
}
