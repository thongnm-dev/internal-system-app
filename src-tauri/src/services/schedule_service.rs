use std::collections::HashMap;
use std::path::Path;

use calamine::{open_workbook, Data, Range, Reader, Xlsx};
use chrono::Datelike;

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::schedule::{DayWork, ScheduleResult, WorkInfoEntry, WorkerSchedule};

pub fn read_schedule(
    file_path: &str,
    target_year: i32,
    target_month: u32,
    user_filter: &str,
) -> AppResult<ScheduleResult> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(AppError::new(format!("File not found: {file_path}")));
    }

    let mut workbook: Xlsx<_> = open_workbook(path)
        .map_err(|e| AppError::new(format!("Cannot open Excel file: {e}")))?;

    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();
    let mut all_workers: HashMap<String, HashMap<String, Vec<WorkInfoEntry>>> = HashMap::new();

    for sheet_name in &sheet_names {
        let range = match workbook.worksheet_range(sheet_name) {
            Ok(r) => r,
            Err(_) => continue,
        };

        if !is_schedule_sheet(&range) {
            continue;
        }

        let last_row = match get_last_row(&range) {
            Some(r) => r.saturating_sub(2),
            None => continue,
        };

        let header_row = match find_header_row(&range) {
            Some(r) => r,
            None => continue,
        };
        let day_row = if header_row > 0 {
            header_row - 1
        } else {
            continue;
        };

        let (month_start_col, month_end_col) =
            match find_month_columns(&range, day_row, header_row, target_year, target_month) {
                Some(cols) => cols,
                None => continue,
            };

        let user_col = match find_column_by_value(&range, header_row, "担当") {
            Some(c) => c,
            None => continue,
        };
        let phase_col = if user_col > 0 { user_col - 1 } else { continue };
        let process_category_col = if phase_col > 0 { phase_col - 1 } else { continue };

        let scr_id_col = match find_column_by_value(&range, header_row, "画面ID") {
            Some(c) => c,
            None => continue,
        };
        let scr_name_col = match find_column_by_value(&range, header_row, "画面名") {
            Some(c) => c,
            None => continue,
        };

        for r in (header_row + 1)..=last_row {
            let current_worker = get_cell_text_merged(&range, r, user_col)
                .trim()
                .to_string();
            let current_worker = if current_worker.is_empty() {
                "No Worker Assigned".to_string()
            } else {
                current_worker
            };

            if !user_filter.is_empty() && current_worker != user_filter {
                continue;
            }

            let bug_id = get_cell_text_merged(&range, r, 0);
            let _job_id = get_cell_text_merged(&range, r, scr_id_col);
            let job_name = get_cell_text_merged(&range, r, scr_name_col);

            for col in month_start_col..=month_end_col {
                let day_text = get_cell_text(&range, header_row, col);
                let day_int: u32 = match day_text.parse() {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                let day_str = format!("{day_int:02}");

                let hours = match get_cell_number(&range, r, col) {
                    Some(h) if h != 0.0 => h,
                    _ => continue,
                };

                let process_category = get_cell_text_merged(&range, r, process_category_col);

                let entry = WorkInfoEntry {
                    phase: map_phase(&process_category),
                    job_id: bug_id.clone(),
                    job_name: job_name.clone(),
                    hours,
                    sheet_name: sheet_name.clone(),
                };

                all_workers
                    .entry(current_worker.clone())
                    .or_default()
                    .entry(day_str)
                    .or_default()
                    .push(entry);
            }
        }
    }

    let mut workers: Vec<WorkerSchedule> = all_workers
        .into_iter()
        .map(|(name, day_map)| {
            let mut days: Vec<DayWork> = day_map
                .into_iter()
                .map(|(day, entries)| DayWork { day, entries })
                .collect();
            days.sort_by(|a, b| a.day.cmp(&b.day));
            WorkerSchedule {
                worker_name: name,
                days,
            }
        })
        .collect();
    workers.sort_by(|a, b| a.worker_name.cmp(&b.worker_name));

    Ok(ScheduleResult {
        file_path: file_path.to_string(),
        target_month: format!("{target_month:02}/{target_year}"),
        workers,
    })
}

fn map_phase(raw: &str) -> String {
    const PHASE_MAP: &[(&str, &str)] = &[
        ("仕様変更", "Thay đổi quy cách"),
        ("バグ", "Bug"),
        ("仕様ミス", "[BUG] Lỗi quy cách"),
    ];
    for &(jp, vi) in PHASE_MAP {
        if raw.contains(jp) {
            return vi.to_string();
        }
    }
    raw.to_string()
}

fn is_schedule_sheet(range: &Range<Data>) -> bool {
    let val = get_cell_text(range, 2, 0);
    val.contains("開発スケジュール") && val.contains("Development Schedule")
}

fn get_last_row(range: &Range<Data>) -> Option<usize> {
    let (rows, _) = range.get_size();
    for r in (0..rows).rev() {
        let text = get_cell_text(range, r, 0);
        if text.starts_with("工程 Operation") {
            return Some(r);
        }
    }
    None
}

fn find_header_row(range: &Range<Data>) -> Option<usize> {
    let (rows, cols) = range.get_size();
    for r in 0..rows {
        for c in 0..cols {
            let text = get_cell_text(range, r, c);
            if text == "No" {
                return Some(r);
            }
        }
    }
    None
}

fn find_column_by_value(range: &Range<Data>, row: usize, value: &str) -> Option<usize> {
    let (_, cols) = range.get_size();
    for c in 0..cols {
        let text = get_cell_text(range, row, c);
        if text.contains(value) {
            return Some(c);
        }
    }
    None
}

fn find_month_columns(
    range: &Range<Data>,
    day_row: usize,
    header_row: usize,
    target_year: i32,
    target_month: u32,
) -> Option<(usize, usize)> {
    let (_, cols) = range.get_size();
    let header_col = find_column_by_value(range, header_row, "No").unwrap_or(0);

    for col in header_col..cols {
        if let Some(cell) = range.get((day_row, col)) {
            if let Some((y, m, _)) = cell_to_ymd(cell) {
                if y == target_year && m == target_month {
                    let days = days_in_month(target_year, target_month) as usize;
                    return Some((col, col + days - 1));
                }
            }
        }
    }
    None
}

fn cell_to_ymd(cell: &Data) -> Option<(i32, u32, u32)> {
    match cell {
        Data::DateTime(dt) => {
            let naive = dt.as_datetime()?;
            Some((naive.year(), naive.month(), naive.day()))
        }
        Data::Float(f) => excel_serial_to_ymd(*f),
        _ => None,
    }
}

fn excel_serial_to_ymd(serial: f64) -> Option<(i32, u32, u32)> {
    if serial < 1.0 {
        return None;
    }
    let days = serial as i64;
    let base = chrono::NaiveDate::from_ymd_opt(1899, 12, 30)?;
    let date = base + chrono::Duration::days(days);
    Some((date.year(), date.month(), date.day()))
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let next = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    let current = chrono::NaiveDate::from_ymd_opt(year, month, 1);
    match (current, next) {
        (Some(c), Some(n)) => (n - c).num_days() as u32,
        _ => 30,
    }
}

fn get_cell_text(range: &Range<Data>, row: usize, col: usize) -> String {
    match range.get((row, col)) {
        Some(Data::String(s)) => s.clone(),
        Some(Data::Float(f)) => {
            if *f == (*f as i64) as f64 {
                format!("{}", *f as i64)
            } else {
                format!("{f}")
            }
        }
        Some(Data::Int(i)) => format!("{i}"),
        Some(Data::Bool(b)) => format!("{b}"),
        Some(Data::DateTime(dt)) => {
            if let Some(naive) = dt.as_datetime() {
                format!("{}", naive.day())
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}

fn get_cell_text_merged(range: &Range<Data>, row: usize, col: usize) -> String {
    let text = get_cell_text(range, row, col);
    if !text.is_empty() {
        return text;
    }
    for r in (0..row).rev() {
        let t = get_cell_text(range, r, col);
        if !t.is_empty() {
            return t;
        }
    }
    String::new()
}

fn get_cell_number(range: &Range<Data>, row: usize, col: usize) -> Option<f64> {
    match range.get((row, col)) {
        Some(Data::Float(f)) => Some(*f),
        Some(Data::Int(i)) => Some(*i as f64),
        Some(Data::String(s)) => s.parse::<f64>().ok(),
        _ => None,
    }
}
