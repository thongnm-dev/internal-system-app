use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectConfig {
    pub input: String,
    pub output: String,
    pub keyword: String,
    pub files: String,
    pub ext: String,
    pub limit_copy: i64,
    pub skip_dir: String,
    pub no_default_skip: bool,
    pub flat: bool,
    pub group_by_parens: bool,
    pub overwrite: bool,
    pub create_history: bool,
    pub delete_non_vn: bool,
    pub report_dir: String,
    pub dry_run: bool,
}

impl Default for CollectConfig {
    fn default() -> Self {
        CollectConfig {
            input: String::new(),
            output: String::new(),
            keyword: String::new(),
            files: String::new(),
            ext: "xlsx".into(),
            limit_copy: 50,
            skip_dir: String::new(),
            no_default_skip: false,
            flat: false,
            group_by_parens: false,
            overwrite: true,
            create_history: true,
            delete_non_vn: false,
            report_dir: "reports".into(),
            dry_run: false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CollectRunResult {
    pub ok: bool,
    pub log: Vec<String>,
    pub summary: String,
}
