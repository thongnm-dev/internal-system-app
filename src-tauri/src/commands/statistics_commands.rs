use crate::domain::statistics::models::AnalysisResult;
use crate::domain::statistics::service;

#[tauri::command]
pub fn analyze_csv(path: String) -> Result<AnalysisResult, String> {
    let path1 = "C:\\Users\\thongnm\\Documents\\check-wr\\csv\\pjjyuji_data_csv_202604.csv";
    service::analyze_csv(&path1).map_err(|error| error.to_string())
}
