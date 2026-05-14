use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiKeyApplication {
    pub id: i64,
    pub application_name: String,
}
