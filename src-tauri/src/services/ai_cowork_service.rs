//! Business logic cho state màn hình AI Cowork.

use crate::app::result::AppResult;
use crate::database::ai_cowork_store::{self, AiCoworkState};

/// Lấy state làm việc gần nhất (project directory đã chọn lần trước).
pub fn get_state() -> AppResult<AiCoworkState> {
    ai_cowork_store::load()
}

/// Lưu lại project directory đang làm việc.
pub fn save_state(project_dir: String) -> AppResult<()> {
    ai_cowork_store::save(&AiCoworkState { project_dir })
}
