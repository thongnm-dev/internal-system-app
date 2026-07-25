//! Business logic cho state màn hình AI Cowork.

use crate::app::result::AppResult;
use crate::database::ai_cowork_store::{self, AiCoworkState};

/// Lấy state làm việc gần nhất (project directory đã chọn lần trước).
pub fn get_state() -> AppResult<AiCoworkState> {
    ai_cowork_store::load()
}

/// Lưu lại toàn bộ state làm việc (project directory, task đang hiển thị, workflow áp dụng).
pub fn save_state(state: AiCoworkState) -> AppResult<()> {
    ai_cowork_store::save(&state)
}
