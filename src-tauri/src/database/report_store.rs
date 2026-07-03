use crate::app::result::AppResult;
use crate::models::import_csv::WorkRecord;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const REPORT_STORE_FILE: &str = "pjjyuji_statistics.json";

#[derive(Default, Deserialize, Serialize)]
pub struct ReportStore {
    pub next_batch_id: i64,
    pub batches: Vec<StoredImportBatch>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct StoredImportBatch {
    pub id: i64,
    pub source_path: String,
    pub source_file_name: String,
    pub imported_at: String,
    pub report_name: String,
    pub note: String,
    pub target_month_from: String,
    pub target_month_to: String,
    pub imported_by: String,
    pub row_count: i64,
    pub total_minutes: i64,
    pub records: Vec<WorkRecord>,
}

impl ReportStore {
    pub fn next_id(&mut self) -> i64 {
        let id = self.next_batch_id.max(1);
        self.next_batch_id = id + 1;
        id
    }
}

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

pub fn save_store(app_data_dir: &Path, store: &ReportStore) -> AppResult<()> {
    let path = store_path(app_data_dir)?;
    let content = serde_json::to_string_pretty(store)?;
    fs::write(path, content)?;
    Ok(())
}

fn store_path(app_data_dir: &Path) -> AppResult<PathBuf> {
    fs::create_dir_all(app_data_dir)?;
    Ok(app_data_dir.join(REPORT_STORE_FILE))
}
