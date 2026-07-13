//! Lưu trữ lịch sử import báo cáo tháng dưới dạng JSON file.
//!
//! File JSON `pjjyuji_statistics.json` nằm trong app data directory,
//! chứa toàn bộ batch đã import kèm dữ liệu work records.

use crate::app::result::AppResult;
use crate::models::import_csv::WorkRecord;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Tên file JSON lưu trữ dữ liệu import.
const REPORT_STORE_FILE: &str = "pjjyuji_statistics.json";

/// Cấu trúc lưu trữ toàn bộ dữ liệu import.
/// Chứa bộ đếm ID tự tăng và danh sách tất cả batch đã import.
#[derive(Default, Deserialize, Serialize)]
pub struct ReportStore {
    /// ID tiếp theo sẽ được cấp cho batch mới.
    pub next_batch_id: i64,
    /// Danh sách tất cả batch đã import.
    pub batches: Vec<StoredImportBatch>,
}

/// Một batch import đã lưu, bao gồm metadata và dữ liệu work records.
#[derive(Clone, Deserialize, Serialize)]
pub struct StoredImportBatch {
    /// ID duy nhất của batch (auto-increment).
    pub id: i64,
    /// Đường dẫn đầy đủ tới file CSV nguồn.
    pub source_path: String,
    /// Tên file CSV.
    pub source_file_name: String,
    /// Thời điểm import.
    pub imported_at: String,
    /// Tên báo cáo do người dùng đặt.
    pub report_name: String,
    /// Ghi chú tùy chọn.
    pub note: String,
    /// Tháng bắt đầu của dữ liệu (YYYY-MM).
    pub target_month_from: String,
    /// Tháng kết thúc của dữ liệu (YYYY-MM).
    pub target_month_to: String,
    /// Người thực hiện import.
    pub imported_by: String,
    /// Tổng số dòng dữ liệu.
    pub row_count: i64,
    /// Tổng số phút làm việc.
    pub total_minutes: i64,
    /// Danh sách bản ghi công việc chi tiết.
    pub records: Vec<WorkRecord>,
}

impl ReportStore {
    /// Cấp ID mới (auto-increment) và tăng bộ đếm lên 1.
    pub fn next_id(&mut self) -> i64 {
        let id = self.next_batch_id.max(1);
        self.next_batch_id = id + 1;
        id
    }
}

/// Đọc store từ file JSON. Tạo store rỗng nếu file chưa tồn tại.
/// Tự động sửa `next_batch_id` nếu bị lệch so với ID lớn nhất trong danh sách.
pub fn load_store(app_data_dir: &Path) -> AppResult<ReportStore> {
    let path = store_path(app_data_dir)?;
    if !path.exists() {
        return Ok(ReportStore {
            next_batch_id: 1,
            batches: Vec::new(),
        });
    }

    let content = fs::read_to_string(path)?;
    let mut store: ReportStore = serde_json::from_str(&content)?;
    // Đảm bảo next_batch_id luôn lớn hơn ID lớn nhất hiện có
    let next_id = store
        .batches
        .iter()
        .map(|batch| batch.id)
        .max()
        .unwrap_or(0)
        + 1;
    store.next_batch_id = store.next_batch_id.max(next_id).max(1);
    Ok(store)
}

/// Ghi store ra file JSON (pretty-printed).
pub fn save_store(app_data_dir: &Path, store: &ReportStore) -> AppResult<()> {
    let path = store_path(app_data_dir)?;
    let content = serde_json::to_string_pretty(store)?;
    fs::write(path, content)?;
    Ok(())
}

/// Xác định đường dẫn file JSON store, tự tạo thư mục nếu chưa có.
fn store_path(app_data_dir: &Path) -> AppResult<PathBuf> {
    fs::create_dir_all(app_data_dir)?;
    Ok(app_data_dir.join(REPORT_STORE_FILE))
}
