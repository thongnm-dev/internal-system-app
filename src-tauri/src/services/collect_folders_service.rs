use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use regex::Regex;

use crate::models::collect::{CollectConfig, CollectRunResult};
use crate::services::collect_service::{
    absolutize, ini_list, is_history_name, norm_ext, DEFAULT_EXTS,
};

fn year_month_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(?:19|20|21)\d{2}(?:0[1-9]|1[0-2])$").unwrap())
}

fn trailing_parens_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\s*[（(][^（）()]*[）)]\s*$").unwrap())
}

fn trailing_number_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[_\-\s]*\d+\s*$").unwrap())
}

fn normalize_folder_name(name: &str) -> String {
    let mut s = name.trim().to_string();
    while let Some(m) = trailing_parens_re().find(&s) {
        s.truncate(m.start());
        s = s.trim_end().to_string();
    }
    s
}

fn base_keyword(normalized: &str) -> String {
    let s = trailing_number_re().replace(normalized.trim(), "");
    s.trim().to_string()
}

fn is_skip_dir_by_folder(name: &str) -> bool {
    let t = name.trim();
    if t.is_empty() {
        return false;
    }
    if year_month_re().is_match(t) {
        return true;
    }
    if t.eq_ignore_ascii_case("bk") {
        return true;
    }
    is_history_name(t)
}

fn parse_range_folder(name: &str) -> Option<(u64, u64)> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"^\s*(\d+)\s*[～〜~\-]\s*(\d+)\s*$").unwrap());
    let c = re.captures(name.trim())?;
    let a: u64 = c.get(1)?.as_str().parse().ok()?;
    let b: u64 = c.get(2)?.as_str().parse().ok()?;
    Some((a.min(b), a.max(b)))
}

fn last_number(normalized: &str) -> Option<u64> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"(\d+)\s*$").unwrap());
    re.captures(normalized.trim())?.get(1)?.as_str().parse().ok()
}

type NameIndex = (HashMap<String, PathBuf>, HashMap<String, PathBuf>);
fn index_children(dir: &Path) -> NameIndex {
    let mut exact: HashMap<String, PathBuf> = HashMap::new();
    let mut norm: HashMap<String, PathBuf> = HashMap::new();
    if let Ok(rd) = fs::read_dir(dir) {
        for p in rd.filter_map(|e| e.ok().map(|x| x.path())) {
            if !p.is_dir() {
                continue;
            }
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            if is_skip_dir_by_folder(&name) {
                continue;
            }
            exact.entry(name.to_lowercase()).or_insert_with(|| p.clone());
            norm.entry(normalize_folder_name(&name).to_lowercase())
                .or_insert(p.clone());
        }
    }
    (exact, norm)
}

fn pick_file_in_folder(
    folder: &Path,
    actual_name: &str,
    normalized: &str,
    exts: &HashSet<String>,
    log: &mut Vec<String>,
) -> Option<PathBuf> {
    let rd = fs::read_dir(folder).ok()?;
    let mut files: Vec<PathBuf> = rd
        .filter_map(|e| e.ok().map(|x| x.path()))
        .filter(|p| p.is_file())
        .filter(|p| {
            let n = p.file_name().and_then(|x| x.to_str()).unwrap_or("");
            if n.starts_with("~$") {
                return false;
            }
            let ext_low = p
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{}", e.to_lowercase()))
                .unwrap_or_default();
            exts.contains(&ext_low)
        })
        .collect();
    files.sort();

    let actual_low = actual_name.to_lowercase();
    let norm_low = normalized.to_lowercase();

    if let Some(hit) = files.iter().find(|p| {
        p.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| {
                let sl = s.to_lowercase();
                sl == actual_low || sl == norm_low
            })
            .unwrap_or(false)
    }) {
        return Some(hit.clone());
    }

    let base = base_keyword(normalized).to_lowercase();
    if base.is_empty() {
        return None;
    }
    let cands: Vec<&PathBuf> = files
        .iter()
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.to_lowercase().contains(&base))
                .unwrap_or(false)
        })
        .collect();
    if cands.len() > 1 {
        log.push(format!(
            "    ⚠ {} file khớp keyword '{}' -> chọn file đầu: {}",
            cands.len(),
            base_keyword(normalized),
            cands[0].file_name().and_then(|n| n.to_str()).unwrap_or("")
        ));
    }
    cands.first().map(|p| (*p).clone())
}

pub fn run(config: CollectConfig) -> Result<CollectRunResult, String> {
    let mut log: Vec<String> = Vec::new();

    if config.input.trim().is_empty() || config.output.trim().is_empty() {
        return Err("Thiếu đường dẫn input/output.".into());
    }

    let targets: Vec<String> = config
        .keyword
        .lines()
        .map(|l| l.trim().trim_start_matches('-').trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    if targets.is_empty() {
        return Err("Chưa nhập danh sách folder ở ô Keyword (mỗi dòng 1 tên).".into());
    }

    let input_root = match fs::canonicalize(&config.input) {
        Ok(p) if p.is_dir() => p,
        _ => {
            return Err(format!(
                "FOLDER_ROOT không tồn tại hoặc không phải thư mục: {}",
                config.input
            ))
        }
    };
    let output_dir = absolutize(&config.output);
    if output_dir == input_root || output_dir.starts_with(&input_root) {
        return Err("Thư mục đích không được nằm trong/đồng nhất FOLDER_ROOT.".into());
    }

    let ext_list = ini_list(&config.ext);
    let mut exts: HashSet<String> = if ext_list.is_empty() {
        DEFAULT_EXTS.iter().map(|s| s.to_string()).collect()
    } else {
        ext_list.iter().map(|e| norm_ext(e)).collect()
    };
    exts.remove("");

    const MAX_DEPTH: usize = 5;
    let mut ranges: Vec<(u64, u64, PathBuf)> = Vec::new();
    let mut stack: Vec<(PathBuf, usize)> = vec![(input_root.clone(), 0)];
    while let Some((current, depth)) = stack.pop() {
        let rd = match fs::read_dir(&current) {
            Ok(rd) => rd,
            Err(e) => {
                log.push(format!("  ⚠ Bỏ qua thư mục không đọc được: {} ({e})", current.display()));
                continue;
            }
        };
        for p in rd.filter_map(|e| e.ok().map(|x| x.path())) {
            if !p.is_dir() {
                continue;
            }
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            if is_skip_dir_by_folder(&name) {
                continue;
            }
            if let Some((a, b)) = parse_range_folder(&name) {
                ranges.push((a, b, p));
                continue;
            }
            if depth + 1 < MAX_DEPTH {
                stack.push((p, depth + 1));
            }
        }
    }
    ranges.sort_by_key(|(a, _, _)| *a);

    let mut range_cache: HashMap<PathBuf, NameIndex> = HashMap::new();

    log.push(format!("FOLDER_ROOT : {}", input_root.display()));
    log.push(format!("OUTPUT      : {}", output_dir.display()));
    log.push("Chế độ      : COPY THEO DANH SÁCH FOLDER".to_string());
    let mut sorted_exts: Vec<&String> = exts.iter().collect();
    sorted_exts.sort();
    log.push(format!("Đuôi file   : {:?}", sorted_exts));
    log.push(format!("Số folder   : {}", targets.len()));
    log.push(format!("Range tìm   : {}", ranges.len()));
    log.push(format!(
        "Chế độ ghi  : {}",
        if config.flat { "FLAT (dồn thẳng vào output)" } else { "output/<tên folder>/" }
    ));
    log.push(format!(
        "Vị trí chính: {}",
        if config.overwrite { "ghi; trùng -> GHI ĐÈ" } else { "ghi nếu chưa có; trùng -> bỏ qua" }
    ));
    if config.dry_run {
        log.push("[DRY-RUN] chỉ xem trước, KHÔNG copy thật.".to_string());
    }
    log.push("-".repeat(60));

    let mut copied = 0usize;
    let mut not_found_folder = 0usize;
    let mut not_found_file = 0usize;
    let mut skipped_exist = 0usize;
    let mut skipped_dup = 0usize;
    let mut seen_flat: HashSet<String> = HashSet::new();

    for target in &targets {
        let target_low = target.to_lowercase();
        let norm_full = normalize_folder_name(target);
        let norm = norm_full.to_lowercase();
        let num = last_number(&norm_full);

        let candidate_ranges: Vec<&PathBuf> = match num {
            Some(n) => {
                let hit: Vec<&PathBuf> = ranges
                    .iter()
                    .filter(|(a, b, _)| n >= *a && n <= *b)
                    .map(|(_, _, p)| p)
                    .collect();
                if hit.is_empty() {
                    ranges.iter().map(|(_, _, p)| p).collect()
                } else {
                    hit
                }
            }
            None => ranges.iter().map(|(_, _, p)| p).collect(),
        };

        let mut folder: Option<PathBuf> = None;
        for range in &candidate_ranges {
            let idx = range_cache
                .entry((*range).clone())
                .or_insert_with(|| index_children(range));
            if let Some(p) = idx.0.get(&target_low).or_else(|| idx.1.get(&norm)) {
                folder = Some(p.clone());
                break;
            }
        }
        let folder = match folder {
            Some(p) => p,
            None => {
                log.push(format!(
                    "  ✖ Không tìm thấy folder: {target}{}",
                    match num {
                        Some(n) if !ranges.iter().any(|(a, b, _)| n >= *a && n <= *b) =>
                            format!("  (số {n} không thuộc range nào)"),
                        _ => String::new(),
                    }
                ));
                not_found_folder += 1;
                continue;
            }
        };
        let actual_name = folder.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let normalized_actual = normalize_folder_name(&actual_name);

        let src = match pick_file_in_folder(&folder, &actual_name, &normalized_actual, &exts, &mut log) {
            Some(p) => p,
            None => {
                log.push(format!(
                    "  ✖ Không thấy file phù hợp trong folder '{}' (tìm cùng tên rồi keyword '{}')",
                    actual_name,
                    base_keyword(&normalized_actual)
                ));
                not_found_file += 1;
                continue;
            }
        };

        let fname = src.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let dest_folder = if config.flat {
            output_dir.clone()
        } else {
            output_dir.join(target)
        };
        let dest = dest_folder.join(&fname);

        if config.flat && !seen_flat.insert(fname.to_lowercase()) {
            log.push(format!("  ⚠ TRÙNG tên (flat), bỏ qua: {} ({target})", fname));
            skipped_dup += 1;
            continue;
        }

        if dest.exists() && !config.overwrite {
            log.push(format!("  ⏭  ĐÃ TỒN TẠI (không ghi đè), bỏ qua: {}", dest.display()));
            skipped_exist += 1;
            continue;
        }

        log.push(format!("  ✔ {}  ->  {}", src.display(), dest.display()));
        if !config.dry_run {
            let _ = fs::create_dir_all(&dest_folder);
            if let Err(e) = fs::copy(&src, &dest) {
                log.push(format!("  ⚠ Không copy được {}: {e}", dest.display()));
            }
        }
        copied += 1;
    }

    log.push("-".repeat(60));
    let verb = if config.dry_run { "Sẽ copy" } else { "Đã copy" };
    let mut summary = format!("{verb}: {copied}/{} folder.", targets.len());
    if not_found_folder > 0 {
        summary.push_str(&format!("  Không thấy folder: {not_found_folder}."));
    }
    if not_found_file > 0 {
        summary.push_str(&format!("  Không thấy file: {not_found_file}."));
    }
    if skipped_dup > 0 {
        summary.push_str(&format!("  Bỏ qua trùng tên: {skipped_dup}."));
    }
    if skipped_exist > 0 {
        summary.push_str(&format!("  Bỏ qua do đã tồn tại: {skipped_exist}."));
    }
    log.push(summary.clone());

    let ok = copied > 0 && not_found_folder == 0 && not_found_file == 0;
    Ok(CollectRunResult { ok, log, summary })
}
