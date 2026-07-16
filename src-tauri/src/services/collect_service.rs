use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use chrono::Local;
use regex::Regex;

use crate::models::collect::{CollectConfig, CollectRunResult};

const VN_SUFFIX: &str = "_VN";
pub(crate) const DEFAULT_EXTS: &[&str] = &[".xlsx"];
const DEFAULT_LIMIT_COPY: i64 = 50;
const INI_SECTION: &str = "collect_input";

const DEFAULT_SKIP_DIR_KEYWORDS: &[&str] = &[
    "履歴", "history", "old", "旧", "backup", "bak", "過去", "archive",
];

const OPEN_PARENS: [char; 2] = ['(', '（'];
const CLOSE_PARENS: [char; 2] = [')', '）'];

fn history_date_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^(?:19|20|21)\d{2}(?:0[1-9]|1[0-2])(?:0[1-9]|[12]\d|3[01])(?:_\d+)?$").unwrap()
    })
}

fn invalid_dir_chars_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"[\\/:*?"<>|]"#).unwrap())
}

pub(crate) fn ini_list(raw: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in raw.lines() {
        for item in line.split(',') {
            let item = item.trim().trim_start_matches('-').trim();
            if !item.is_empty() {
                out.push(item.to_string());
            }
        }
    }
    out
}

fn parse_bool(raw: &str, default: bool) -> bool {
    let s = raw.trim().to_lowercase();
    if s.is_empty() {
        return default;
    }
    matches!(s.as_str(), "1" | "true" | "yes" | "on" | "y")
}

pub(crate) fn norm_ext(ext: &str) -> String {
    let ext = ext.trim().to_lowercase();
    if ext.is_empty() {
        return String::new();
    }
    if ext.starts_with('.') {
        ext
    } else {
        format!(".{ext}")
    }
}

fn is_skip_dir(name: &str, skip_keywords: &[String], match_date_history: bool) -> bool {
    if match_date_history && history_date_re().is_match(name.trim()) {
        return true;
    }
    let low = name.to_lowercase();
    skip_keywords.iter().any(|kw| low.contains(&kw.to_lowercase()))
}

pub(crate) fn is_history_name(name: &str) -> bool {
    history_date_re().is_match(name.trim())
        || DEFAULT_SKIP_DIR_KEYWORDS
            .iter()
            .any(|kw| name.to_lowercase().contains(&kw.to_lowercase()))
}

fn build_non_vn_index(output_dir: &Path) -> HashMap<String, Vec<PathBuf>> {
    let mut map: HashMap<String, Vec<PathBuf>> = HashMap::new();
    if !output_dir.is_dir() {
        return map;
    }
    let mut stack = vec![output_dir.to_path_buf()];
    while let Some(cur) = stack.pop() {
        let rd = match fs::read_dir(&cur) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for entry in rd.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_dir() {
                let n = p.file_name().and_then(|x| x.to_str()).unwrap_or("");
                if is_history_name(n) {
                    continue;
                }
                stack.push(p);
            } else if p.is_file() {
                if let Some(fname) = p.file_name().and_then(|x| x.to_str()) {
                    map.entry(fname.to_lowercase()).or_default().push(p.clone());
                }
            }
        }
    }
    map
}

fn match_file(
    path: &Path,
    exts: &HashSet<String>,
    keywords: &[String],
    files_set: &HashSet<String>,
) -> bool {
    let name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return false,
    };
    if name.starts_with("~$") {
        return false;
    }
    let name_low = name.to_lowercase();
    if !files_set.is_empty() {
        return files_set.contains(&name_low);
    }
    let ext_low = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e.to_lowercase()))
        .unwrap_or_default();
    if !exts.contains(&ext_low) {
        return false;
    }
    if keywords.is_empty() {
        return true;
    }
    keywords.iter().any(|kw| name_low.contains(&kw.to_lowercase()))
}

fn folder_from_parens(filename: &str) -> Option<String> {
    let chars: Vec<char> = filename.chars().collect();
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut depth: i32 = 0;
    let mut start: usize = 0;
    for (i, &ch) in chars.iter().enumerate() {
        if OPEN_PARENS.contains(&ch) {
            if depth == 0 {
                start = i + 1;
            }
            depth += 1;
        } else if CLOSE_PARENS.contains(&ch) && depth > 0 {
            depth -= 1;
            if depth == 0 {
                spans.push((start, i));
            }
        }
    }
    let (s, e) = *spans.last()?;
    let content: String = chars[s..e].iter().collect();
    let cleaned = invalid_dir_chars_re()
        .replace_all(content.trim(), "_")
        .trim()
        .to_string();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn non_vn_name(filename: &str) -> Option<String> {
    let p = Path::new(filename);
    let stem = p.file_stem()?.to_str()?;
    if stem.to_lowercase().ends_with(&VN_SUFFIX.to_lowercase()) {
        let base = &stem[..stem.len() - VN_SUFFIX.len()];
        match p.extension().and_then(|e| e.to_str()) {
            Some(ext) => Some(format!("{base}.{ext}")),
            None => Some(base.to_string()),
        }
    } else {
        None
    }
}

fn next_history_dir(parent: &Path, today: &str) -> PathBuf {
    let candidate = parent.join(today);
    if !candidate.exists() {
        return candidate;
    }
    let mut n = 2;
    loop {
        let candidate = parent.join(format!("{today}_{n:02}"));
        if !candidate.exists() {
            return candidate;
        }
        n += 1;
    }
}

pub(crate) fn absolutize(p: &str) -> PathBuf {
    let pb = PathBuf::from(p);
    if pb.is_absolute() {
        pb
    } else {
        std::env::current_dir().unwrap_or_default().join(pb)
    }
}

fn find_repo_root() -> Option<PathBuf> {
    let mut starts: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        starts.push(cwd);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            starts.push(dir.to_path_buf());
        }
    }
    for start in starts {
        let mut cur: Option<&Path> = Some(start.as_path());
        while let Some(dir) = cur {
            if dir.join("tools").join("collect_input.ini").is_file() {
                return Some(dir.to_path_buf());
            }
            cur = dir.parent();
        }
    }
    None
}

fn parse_ini_section(text: &str) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    let mut in_section = false;
    let mut last_key: Option<String> = None;
    for raw_line in text.lines() {
        let trimmed = raw_line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let name = trimmed[1..trimmed.len() - 1].trim();
            in_section = name == INI_SECTION;
            last_key = None;
            continue;
        }
        if !in_section {
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }
        let indented = raw_line.starts_with(' ') || raw_line.starts_with('\t');
        if indented && last_key.is_some() && !trimmed.contains('=') {
            if let Some(k) = &last_key {
                let entry = map.entry(k.clone()).or_default();
                if !entry.is_empty() {
                    entry.push('\n');
                }
                entry.push_str(trimmed);
            }
            continue;
        }
        if let Some(eq) = raw_line.find('=') {
            let key = raw_line[..eq].trim().to_string();
            let val = raw_line[eq + 1..].trim().to_string();
            last_key = Some(key.clone());
            map.insert(key, val);
        }
    }
    map
}

pub fn load_ini() -> Result<CollectConfig, String> {
    let root = find_repo_root()
        .ok_or_else(|| "Không tìm thấy tools/collect_input.ini (đi ngược từ thư mục hiện tại).".to_string())?;
    let ini_path = root.join("tools").join("collect_input.ini");
    let text = fs::read_to_string(&ini_path)
        .map_err(|e| format!("Không đọc được {}: {e}", ini_path.display()))?;
    let m = parse_ini_section(&text);

    let get = |k: &str| m.get(k).cloned().unwrap_or_default();
    let get_bool = |k: &str, default: bool| match m.get(k) {
        Some(v) => parse_bool(v, default),
        None => default,
    };
    let limit_copy = match m.get("limit_copy") {
        Some(v) if !v.trim().is_empty() => v.trim().parse::<i64>().unwrap_or(DEFAULT_LIMIT_COPY),
        _ => DEFAULT_LIMIT_COPY,
    };
    let ext = {
        let raw = get("ext");
        if raw.trim().is_empty() {
            "xlsx".to_string()
        } else {
            raw
        }
    };
    let report_dir = {
        let raw = get("report_dir");
        if raw.trim().is_empty() {
            "reports".to_string()
        } else {
            raw
        }
    };

    Ok(CollectConfig {
        input: get("input"),
        output: get("output"),
        keyword: get("keyword"),
        files: get("files"),
        ext,
        limit_copy,
        skip_dir: get("skip_dir"),
        no_default_skip: get_bool("no_default_skip", false),
        flat: get_bool("flat", false),
        group_by_parens: get_bool("group_by_parens", false),
        overwrite: get_bool("overwrite", true),
        create_history: get_bool("create_history", true),
        delete_non_vn: get_bool("delete_non_vn", false),
        report_dir,
        dry_run: get_bool("dry_run", false),
    })
}

struct CollectOut {
    copied: usize,
    skipped_dup: usize,
    skipped_exist: Vec<(PathBuf, PathBuf)>,
    deleted_non_vn: Vec<PathBuf>,
    history_copied: Vec<(PathBuf, PathBuf)>,
}

#[allow(clippy::too_many_arguments)]
fn collect(
    input_root: &Path,
    output_dir: &Path,
    keywords: &[String],
    exts: &HashSet<String>,
    skip_dir_keywords: &[String],
    flat: bool,
    dry_run: bool,
    files: &[String],
    limit_copy: i64,
    match_date_history: bool,
    group_by_parens: bool,
    overwrite: bool,
    delete_non_vn: bool,
    create_history: bool,
    log: &mut Vec<String>,
) -> CollectOut {
    let mut copied = 0usize;
    let mut skipped_dup = 0usize;
    let mut skipped_exist: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut deleted_non_vn: Vec<PathBuf> = Vec::new();
    let mut history_copied: Vec<(PathBuf, PathBuf)> = Vec::new();
    let today = Local::now().format("%Y%m%d").to_string();
    let mut history_dirs: HashMap<PathBuf, PathBuf> = HashMap::new();
    let mut seen_flat_names: HashSet<String> = HashSet::new();
    let mut seen_dest: HashSet<PathBuf> = HashSet::new();
    let files_set: HashSet<String> = files.iter().map(|f| f.to_lowercase()).collect();

    let mut stack: Vec<PathBuf> = vec![input_root.to_path_buf()];
    let mut matched: Vec<PathBuf> = Vec::new();
    while let Some(current) = stack.pop() {
        let entries = match fs::read_dir(&current) {
            Ok(rd) => rd,
            Err(e) => {
                log.push(format!("  ⚠ Bỏ qua thư mục không đọc được: {} ({e})", current.display()));
                continue;
            }
        };
        let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|x| x.path())).collect();
        paths.sort();
        for entry in paths {
            if entry.is_dir() {
                let name = entry.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                if is_skip_dir(&name, skip_dir_keywords, match_date_history) {
                    let rel = entry.strip_prefix(input_root).unwrap_or(&entry);
                    log.push(format!("  ⏭  Bỏ qua thư mục lịch sử: {}", rel.display()));
                    continue;
                }
                stack.push(entry);
            } else if entry.is_file() && match_file(&entry, exts, keywords, &files_set) {
                matched.push(entry);
            }
        }
    }

    matched.sort();

    let non_vn_index: HashMap<String, Vec<PathBuf>> = if delete_non_vn {
        build_non_vn_index(output_dir)
    } else {
        HashMap::new()
    };

    if !files_set.is_empty() {
        let found: HashSet<String> = matched
            .iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(|s| s.to_lowercase()))
            .collect();
        let missing: Vec<&String> = files.iter().filter(|f| !found.contains(&f.to_lowercase())).collect();
        if !missing.is_empty() {
            log.push(format!(
                "  ⚠ Không tìm thấy {} file đã khai trong 'files': {:?}",
                missing.len(),
                missing
            ));
        }
    } else if limit_copy > 0 && matched.len() as i64 > limit_copy {
        log.push(format!(
            "  ⚠ Khớp {} file > giới hạn limit_copy={} -> chỉ copy {} file đầu (theo thứ tự đường dẫn). \
             Tăng limit_copy hoặc dùng 'files'/'keyword' hẹp hơn.",
            matched.len(),
            limit_copy,
            limit_copy
        ));
        matched.truncate(limit_copy as usize);
    }

    for src in &matched {
        let rel = src.strip_prefix(input_root).unwrap_or(src).to_path_buf();
        let name = src.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();

        let dest: PathBuf;
        if group_by_parens {
            match folder_from_parens(&name) {
                Some(folder) => dest = output_dir.join(folder).join(&name),
                None => {
                    dest = output_dir.join(&name);
                    log.push(format!("  ⚠ Không thấy '()' trong tên -> để ở gốc: {name}"));
                }
            }
            if seen_dest.contains(&dest) {
                log.push(format!("  ⚠ TRÙNG đích, bỏ qua: {} -> {}", rel.display(), dest.display()));
                skipped_dup += 1;
                continue;
            }
            seen_dest.insert(dest.clone());
        } else if flat {
            dest = output_dir.join(&name);
            if seen_flat_names.contains(&name) {
                log.push(format!("  ⚠ TRÙNG tên (flat), bỏ qua: {}", rel.display()));
                skipped_dup += 1;
                continue;
            }
            seen_flat_names.insert(name.clone());
        } else {
            dest = output_dir.join(&rel);
        }

        let dest_parent = dest.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| output_dir.to_path_buf());

        if delete_non_vn {
            if let Some(non_vn) = non_vn_name(&name) {
                if let Some(victims) = non_vn_index.get(&non_vn.to_lowercase()) {
                    for victim in victims {
                        if victim.exists() && *victim != dest {
                            log.push(format!("  🗑  Xóa bản gốc (non-VN): {}", victim.display()));
                            if !dry_run {
                                if let Err(e) = fs::remove_file(victim) {
                                    log.push(format!("  ⚠ Không xóa được {}: {e}", victim.display()));
                                }
                            }
                            deleted_non_vn.push(victim.clone());
                        }
                    }
                }
            }
        }

        if create_history {
            let hist_dir = history_dirs
                .entry(dest_parent.clone())
                .or_insert_with(|| next_history_dir(&dest_parent, &today))
                .clone();
            let hist_dest = hist_dir.join(&name);
            log.push(format!("  🕒 {}  ->  {}", rel.display(), hist_dest.display()));
            if !dry_run {
                let _ = fs::create_dir_all(&hist_dir);
                if let Err(e) = fs::copy(src, &hist_dest) {
                    log.push(format!("  ⚠ Không copy được snapshot {}: {e}", hist_dest.display()));
                }
            }
            history_copied.push((src.clone(), hist_dest));
        }

        if dest.exists() && !overwrite {
            if create_history {
                log.push(format!("  ↳ vị trí chính giữ nguyên (đã tồn tại, không ghi đè): {}", dest.display()));
            } else {
                log.push(format!("  ⏭  ĐÃ TỒN TẠI (không ghi đè), bỏ qua: {}", dest.display()));
                skipped_exist.push((src.clone(), dest.clone()));
            }
        } else {
            log.push(format!("  ✔ {}  ->  {}", rel.display(), dest.display()));
            if !dry_run {
                let _ = fs::create_dir_all(&dest_parent);
                if let Err(e) = fs::copy(src, &dest) {
                    log.push(format!("  ⚠ Không copy được {}: {e}", dest.display()));
                }
            }
            copied += 1;
        }
    }

    CollectOut {
        copied,
        skipped_dup,
        skipped_exist,
        deleted_non_vn,
        history_copied,
    }
}

fn write_existing_report(report_dir: &Path, skipped: &[(PathBuf, PathBuf)], stamp: &str) -> Result<PathBuf, String> {
    fs::create_dir_all(report_dir).map_err(|e| format!("Không tạo được {}: {e}", report_dir.display()))?;
    let out = report_dir.join(format!("collect_input_existing_{stamp}.tsv"));
    let mut lines = vec!["source\tdestination".to_string()];
    for (src, dest) in skipped {
        lines.push(format!("{}\t{}", src.display(), dest.display()));
    }
    let content = lines.join("\n") + "\n";
    fs::write(&out, content).map_err(|e| format!("Không ghi được {}: {e}", out.display()))?;
    Ok(out)
}

pub fn run(config: CollectConfig) -> Result<CollectRunResult, String> {
    let mut log: Vec<String> = Vec::new();

    if config.input.trim().is_empty() || config.output.trim().is_empty() {
        return Err("Thiếu đường dẫn input/output.".into());
    }

    let files = ini_list(&config.files);
    let keywords = ini_list(&config.keyword);
    let ext_list = ini_list(&config.ext);
    let skip_dir_extra = ini_list(&config.skip_dir);

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

    let mut exts: HashSet<String> = if ext_list.is_empty() {
        DEFAULT_EXTS.iter().map(|s| s.to_string()).collect()
    } else {
        ext_list.iter().map(|e| norm_ext(e)).collect()
    };
    exts.remove("");

    let mut skip_keywords: Vec<String> = if config.no_default_skip {
        Vec::new()
    } else {
        DEFAULT_SKIP_DIR_KEYWORDS.iter().map(|s| s.to_string()).collect()
    };
    skip_keywords.extend(skip_dir_extra);
    let match_date_history = !config.no_default_skip;

    log.push(format!("FOLDER_ROOT : {}", input_root.display()));
    log.push(format!("OUTPUT      : {}", output_dir.display()));
    if !files.is_empty() {
        log.push(format!(
            "Chế độ lọc  : FILES (danh sách cố định, {} tên — bỏ qua keyword/ext/limit)",
            files.len()
        ));
        log.push(format!("Files       : {:?}", files));
    } else {
        log.push("Chế độ lọc  : KEYWORD".to_string());
        let mut sorted_exts: Vec<&String> = exts.iter().collect();
        sorted_exts.sort();
        log.push(format!("Đuôi file   : {:?}", sorted_exts));
        log.push(format!(
            "Keyword     : {}",
            if keywords.is_empty() { "(tất cả)".to_string() } else { format!("{:?}", keywords) }
        ));
        log.push(format!(
            "Giới hạn    : {}",
            if config.limit_copy > 0 { config.limit_copy.to_string() } else { "(không giới hạn)".to_string() }
        ));
    }
    log.push(format!(
        "Bỏ qua dir  : {}{}",
        if skip_keywords.is_empty() { "(không)".to_string() } else { format!("{:?}", skip_keywords) },
        if match_date_history { "  + thư mục ngày yyyyMMdd[_N]" } else { "" }
    ));
    let write_mode = if config.group_by_parens {
        "thư mục con theo nội dung trong ()"
    } else if config.flat {
        "FLAT"
    } else {
        "giữ cấu trúc con"
    };
    log.push(format!(
        "Chế độ ghi  : {}{}",
        write_mode,
        if config.dry_run { "  [DRY-RUN]" } else { "" }
    ));
    log.push(format!(
        "Lịch sử     : {}",
        if config.create_history { "CÓ - luôn copy snapshot vào yyyyMMdd[_NN]" } else { "KHÔNG" }
    ));
    if config.overwrite {
        log.push("Vị trí chính: ghi; trùng -> GHI ĐÈ".to_string());
    } else if config.create_history {
        log.push("Vị trí chính: ghi nếu chưa có; trùng -> giữ nguyên (chỉ vào lịch sử)".to_string());
    } else {
        log.push("Vị trí chính: ghi nếu chưa có; trùng -> bỏ qua + log reports/".to_string());
    }
    if config.delete_non_vn {
        log.push("Xóa non-VN  : CÓ (xóa bản gốc cùng tên không có '_VN')".to_string());
    }
    log.push("-".repeat(60));

    let res = collect(
        &input_root,
        &output_dir,
        &keywords,
        &exts,
        &skip_keywords,
        config.flat,
        config.dry_run,
        &files,
        config.limit_copy,
        match_date_history,
        config.group_by_parens,
        config.overwrite,
        config.delete_non_vn,
        config.create_history,
        &mut log,
    );

    log.push("-".repeat(60));
    let verb = if config.dry_run { "Sẽ ghi" } else { "Đã ghi" };
    let mut summary = format!(
        "{verb} -> vị trí chính: {}, lịch sử: {}.",
        res.copied,
        res.history_copied.len()
    );
    if res.skipped_dup > 0 {
        summary.push_str(&format!("  Bỏ qua trùng tên: {}.", res.skipped_dup));
    }
    if !res.deleted_non_vn.is_empty() {
        summary.push_str(&format!("  Đã xóa bản non-VN: {}.", res.deleted_non_vn.len()));
    }
    if !res.skipped_exist.is_empty() {
        summary.push_str(&format!("  Bỏ qua do đã tồn tại: {}.", res.skipped_exist.len()));
    }
    log.push(summary.clone());

    if !res.skipped_exist.is_empty() {
        let report_root = find_repo_root().unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        let rd = PathBuf::from(&config.report_dir);
        let report_dir = if rd.is_absolute() { rd } else { report_root.join(rd) };
        let stamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        match write_existing_report(&report_dir, &res.skipped_exist, &stamp) {
            Ok(out) => log.push(format!("📄 Log {} file đã tồn tại -> {}", res.skipped_exist.len(), out.display())),
            Err(e) => log.push(format!("⚠ Không ghi được report: {e}")),
        }
    }

    let ok = !(res.copied == 0 && res.skipped_exist.is_empty() && res.history_copied.is_empty());
    if !ok {
        log.push("⚠ Không có file nào khớp. Kiểm tra lại files / keyword / ext / FOLDER_ROOT.".to_string());
    }

    Ok(CollectRunResult { ok, log, summary })
}
