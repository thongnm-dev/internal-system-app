use crate::utils::app_config::config_path;
use ini::Ini;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct PaginationConfig {
    pub rows: u32,
    pub rows_per_page_options: Vec<u32>,
    pub rows_compact: u32,
    pub rows_per_page_options_compact: Vec<u32>,
    pub paginator_template: String,
    pub current_page_report_template: String,
}

fn parse_int_list(raw: &str) -> Vec<u32> {
    raw.split(',')
        .filter_map(|s| s.trim().parse::<u32>().ok())
        .filter(|&n| n > 0)
        .collect()
}

pub fn get_pagination_config() -> PaginationConfig {
    let default = PaginationConfig {
        rows: 20,
        rows_per_page_options: vec![20, 50, 100],
        rows_compact: 10,
        rows_per_page_options_compact: vec![10, 20, 50],
        paginator_template: "FirstPageLink PrevPageLink PageLinks NextPageLink LastPageLink RowsPerPageDropdown CurrentPageReport".to_string(),
        current_page_report_template: "Showing {first} to {last} of {totalRecords}".to_string(),
    };

    let path = config_path();
    let ini = match Ini::load_from_file(&path) {
        Ok(ini) => ini,
        Err(_) => return default,
    };

    let section = match ini.section(Some("pagination")) {
        Some(s) => s,
        None => return default,
    };

    let rows = section
        .get("ROWS")
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default.rows);

    let rows_per_page_options = section
        .get("ROWS_PER_PAGE_OPTIONS")
        .map(parse_int_list)
        .filter(|v| !v.is_empty())
        .unwrap_or(default.rows_per_page_options);

    let rows_compact = section
        .get("ROWS_COMPACT")
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default.rows_compact);

    let rows_per_page_options_compact = section
        .get("ROWS_PER_PAGE_OPTIONS_COMPACT")
        .map(parse_int_list)
        .filter(|v| !v.is_empty())
        .unwrap_or(default.rows_per_page_options_compact);

    let paginator_template = section
        .get("PAGINATOR_TEMPLATE")
        .filter(|v| !v.trim().is_empty())
        .map(|v| v.to_string())
        .unwrap_or(default.paginator_template);

    let current_page_report_template = section
        .get("CURRENT_PAGE_REPORT_TEMPLATE")
        .filter(|v| !v.trim().is_empty())
        .map(|v| v.to_string())
        .unwrap_or(default.current_page_report_template);

    PaginationConfig {
        rows,
        rows_per_page_options,
        rows_compact,
        rows_per_page_options_compact,
        paginator_template,
        current_page_report_template,
    }
}
