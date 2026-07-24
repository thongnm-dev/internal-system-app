//! Business logic cho state màn hình AI Translate Cowork.

use crate::app::result::AppResult;
use crate::database::ai_translate_cowork_store::{self, AiTranslateCoworkState};

/// Lấy state làm việc gần nhất (project directory đã chọn lần trước).
pub fn get_state() -> AppResult<AiTranslateCoworkState> {
    ai_translate_cowork_store::load()
}

/// Lưu lại project directory đang làm việc.
pub fn save_state(project_dir: String) -> AppResult<()> {
    ai_translate_cowork_store::save(&AiTranslateCoworkState { project_dir })
}
