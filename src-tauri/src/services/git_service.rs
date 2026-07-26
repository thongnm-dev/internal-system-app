//! Service cho module Git Desktop.
//!
//! Toàn bộ thao tác Git được thực hiện bằng cách gọi `git` CLI của hệ điều hành
//! (giống cách GitHub Desktop hoạt động). Cách này:
//! - Tận dụng credential helper / cấu hình git sẵn có của người dùng nên
//!   fetch/pull/push "chạy được ngay" mà không cần quản lý SSH key/token.
//! - Không cần thêm dependency native nặng (libgit2).
//!
//! Các thao tác chạm mạng (fetch/pull/push/clone) và thao tác ghi nặng được gọi
//! qua `spawn_blocking` ở tầng command để không chặn UI.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use reqwest::Client;

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::models::git::{
    GitBranch, GitCommit, GitCommitDetail, GitComparison, GitDiff, GitDiffLine, GitFileChange,
    GitProgress, GitPullRequest, GitRepoInfo, GitStash, GitStatus, GitTag, GitWorktree,
};

/// Giới hạn số dòng diff trả về để tránh treo UI với file cực lớn.
const MAX_DIFF_LINES: usize = 6000;
/// Giới hạn kích thước file untracked khi dựng diff tổng hợp (2MB).
const MAX_UNTRACKED_DIFF_SIZE: u64 = 2 * 1024 * 1024;

/// Ký tự phân tách field/record khi format output của git (ít khả năng xuất hiện trong nội dung).
const FS: char = '\u{1f}'; // field separator
const RS: char = '\u{1e}'; // record separator

/// Tạo `Command` git đã cấu hình sẵn: chạy trong `repo_path`, tắt prompt tương tác
/// (tránh treo khi thiếu credential), và ẩn cửa sổ console trên Windows.
fn git_in(repo_path: &str) -> Command {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(repo_path);
    configure(&mut cmd);
    cmd
}

/// Cấu hình chung cho mọi lệnh git.
fn configure(cmd: &mut Command) {
    // Không bao giờ bật prompt tương tác — nếu thiếu credential thì fail luôn
    // thay vì treo app chờ nhập.
    cmd.env("GIT_TERMINAL_PROMPT", "0");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
}

/// Chạy một lệnh git trong `repo_path` và trả về stdout (đã trim CR/LF cuối).
/// Lỗi (exit code != 0) trả về `AppError` chứa stderr.
fn run(repo_path: &str, args: &[&str]) -> AppResult<String> {
    let output = git_in(repo_path)
        .args(args)
        .output()
        .map_err(|e| AppError::new(format!("Không chạy được git: {e}. Hãy chắc chắn đã cài Git.")))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let msg = if !stderr.is_empty() { stderr } else { stdout };
        Err(AppError::new(if msg.is_empty() {
            "Lệnh git thất bại.".to_string()
        } else {
            msg
        }))
    }
}

/// Kiểm tra một đường dẫn có phải thư mục làm việc của git repo không.
pub fn is_git_repo(path: &str) -> bool {
    if !Path::new(path).is_dir() {
        return false;
    }
    git_in(path)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Chạy một lệnh, coi lỗi là "không có" và trả về chuỗi rỗng (dùng cho các query
/// tùy chọn như upstream/remote có thể chưa tồn tại).
fn run_opt(repo_path: &str, args: &[&str]) -> String {
    run(repo_path, args).unwrap_or_default().trim().to_string()
}

/// Lấy trạng thái tổng quan của repo.
pub fn repo_info(repo_path: &str) -> AppResult<GitRepoInfo> {
    if !is_git_repo(repo_path) {
        return Err(AppError::new(format!(
            "Không phải Git repository: {repo_path}"
        )));
    }

    let top_level = run_opt(repo_path, &["rev-parse", "--show-toplevel"]);
    let path = if top_level.is_empty() {
        repo_path.to_string()
    } else {
        top_level
    };

    let branch_ref = run(repo_path, &["symbolic-ref", "--short", "-q", "HEAD"]).ok();
    let (current_branch, detached) = match branch_ref {
        Some(b) if !b.trim().is_empty() => (b.trim().to_string(), false),
        _ => {
            let short = run_opt(repo_path, &["rev-parse", "--short", "HEAD"]);
            (short, true)
        }
    };

    let upstream = run_opt(
        repo_path,
        &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
    );

    let (mut ahead, mut behind) = (0u32, 0u32);
    if !upstream.is_empty() {
        // Output: "<behind>\t<ahead>" (left = @{u}, right = HEAD).
        let counts = run_opt(
            repo_path,
            &["rev-list", "--left-right", "--count", "@{u}...HEAD"],
        );
        let mut parts = counts.split_whitespace();
        behind = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        ahead = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    }

    let remote_url = run_opt(repo_path, &["remote", "get-url", "origin"]);

    let (rebase_in_progress, cherry_pick_in_progress, merge_in_progress) = {
        let git_dir = run_opt(repo_path, &["rev-parse", "--git-dir"]);
        if git_dir.is_empty() {
            (false, false, false)
        } else {
            let base = Path::new(&git_dir);
            let dir = if base.is_absolute() {
                base.to_path_buf()
            } else {
                Path::new(repo_path).join(base)
            };
            let rebase =
                dir.join("rebase-merge").exists() || dir.join("rebase-apply").exists();
            let cherry = dir.join("CHERRY_PICK_HEAD").exists();
            let merge = dir.join("MERGE_HEAD").exists();
            (rebase, cherry, merge)
        }
    };

    Ok(GitRepoInfo {
        path,
        current_branch,
        detached,
        upstream,
        ahead,
        behind,
        remote_url,
        rebase_in_progress,
        cherry_pick_in_progress,
        merge_in_progress,
    })
}

/// Lấy danh sách thay đổi (staged + unstaged), parse từ `git status --porcelain -z`.
pub fn status(repo_path: &str) -> AppResult<GitStatus> {
    let raw = run(
        repo_path,
        &[
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
            "-z",
        ],
    )?;

    let tokens: Vec<&str> = raw.split('\0').collect();
    let mut staged: Vec<GitFileChange> = Vec::new();
    let mut unstaged: Vec<GitFileChange> = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        let entry = tokens[i];
        if entry.len() < 3 {
            i += 1;
            continue;
        }
        let bytes = entry.as_bytes();
        let x = bytes[0] as char;
        let y = bytes[1] as char;
        let path = entry[3..].to_string();

        // Rename/copy: token kế tiếp là đường dẫn gốc.
        let mut orig = String::new();
        if x == 'R' || x == 'C' || y == 'R' || y == 'C' {
            i += 1;
            if i < tokens.len() {
                orig = tokens[i].to_string();
            }
        }

        if x == '?' && y == '?' {
            unstaged.push(GitFileChange {
                path,
                orig_path: String::new(),
                status: "?".to_string(),
                untracked: true,
            });
        } else {
            if x != ' ' && x != '?' {
                staged.push(GitFileChange {
                    path: path.clone(),
                    orig_path: orig.clone(),
                    status: x.to_string(),
                    untracked: false,
                });
            }
            if y != ' ' && y != '?' {
                unstaged.push(GitFileChange {
                    path,
                    orig_path: orig,
                    status: y.to_string(),
                    untracked: false,
                });
            }
        }
        i += 1;
    }

    Ok(GitStatus { staged, unstaged })
}

/// Parse output unified diff thành danh sách dòng có phân loại.
fn parse_diff(path: &str, raw: &str) -> GitDiff {
    let mut lines: Vec<GitDiffLine> = Vec::new();
    let mut is_binary = false;
    let mut truncated = false;
    let (mut old_ln, mut new_ln) = (0u32, 0u32);

    for line in raw.lines() {
        if lines.len() >= MAX_DIFF_LINES {
            truncated = true;
            break;
        }

        if line.starts_with("Binary files") || line.starts_with("GIT binary patch") {
            is_binary = true;
            continue;
        }
        if line.starts_with("diff --git")
            || line.starts_with("index ")
            || line.starts_with("--- ")
            || line.starts_with("+++ ")
            || line.starts_with("new file")
            || line.starts_with("deleted file")
            || line.starts_with("old mode")
            || line.starts_with("new mode")
            || line.starts_with("similarity")
            || line.starts_with("rename ")
            || line.starts_with("copy ")
            || line.starts_with("\\ No newline")
        {
            continue;
        }

        if line.starts_with("@@") {
            // @@ -oldStart,oldCount +newStart,newCount @@ context
            if let Some((os, ns)) = parse_hunk_header(line) {
                old_ln = os;
                new_ln = ns;
            }
            lines.push(GitDiffLine {
                kind: "hunk".to_string(),
                content: line.to_string(),
                old_line: 0,
                new_line: 0,
            });
            continue;
        }

        let first = line.chars().next().unwrap_or(' ');
        match first {
            '+' => {
                lines.push(GitDiffLine {
                    kind: "add".to_string(),
                    content: line[1..].to_string(),
                    old_line: 0,
                    new_line: new_ln,
                });
                new_ln += 1;
            }
            '-' => {
                lines.push(GitDiffLine {
                    kind: "del".to_string(),
                    content: line[1..].to_string(),
                    old_line: old_ln,
                    new_line: 0,
                });
                old_ln += 1;
            }
            _ => {
                let content = if line.is_empty() { "" } else { &line[1..] };
                lines.push(GitDiffLine {
                    kind: "context".to_string(),
                    content: content.to_string(),
                    old_line: old_ln,
                    new_line: new_ln,
                });
                old_ln += 1;
                new_ln += 1;
            }
        }
    }

    GitDiff {
        path: path.to_string(),
        lines,
        is_binary,
        truncated,
    }
}

/// Parse dòng hunk header `@@ -a,b +c,d @@` → (a, c).
fn parse_hunk_header(line: &str) -> Option<(u32, u32)> {
    let inner = line.strip_prefix("@@ ")?;
    let end = inner.find(" @@")?;
    let ranges = &inner[..end];
    let mut parts = ranges.split(' ');
    let old_part = parts.next()?.trim_start_matches('-');
    let new_part = parts.next()?.trim_start_matches('+');
    let old_start = old_part.split(',').next()?.parse().ok()?;
    let new_start = new_part.split(',').next()?.parse().ok()?;
    Some((old_start, new_start))
}

/// Diff của một file trong working tree (unstaged) hoặc index (staged).
pub fn file_diff(repo_path: &str, file: &str, staged: bool, untracked: bool) -> AppResult<GitDiff> {
    if untracked {
        return untracked_diff(repo_path, file);
    }
    let mut args = vec!["diff", "--no-color"];
    if staged {
        args.push("--cached");
    }
    args.push("--");
    args.push(file);
    let raw = run(repo_path, &args)?;
    Ok(parse_diff(file, &raw))
}

/// Dựng diff "tất cả là thêm mới" cho file untracked (git diff không hiển thị file mới).
fn untracked_diff(repo_path: &str, file: &str) -> AppResult<GitDiff> {
    let abs = Path::new(repo_path).join(file);
    let meta = std::fs::metadata(&abs)
        .map_err(|e| AppError::new(format!("Không đọc được file: {e}")))?;
    if meta.len() > MAX_UNTRACKED_DIFF_SIZE {
        return Ok(GitDiff {
            path: file.to_string(),
            lines: vec![],
            is_binary: false,
            truncated: true,
        });
    }
    let bytes = std::fs::read(&abs).map_err(|e| AppError::new(format!("Không đọc được file: {e}")))?;
    if bytes.contains(&0) {
        return Ok(GitDiff {
            path: file.to_string(),
            lines: vec![],
            is_binary: true,
            truncated: false,
        });
    }
    let text = String::from_utf8_lossy(&bytes);
    let mut lines: Vec<GitDiffLine> = Vec::new();
    let mut new_ln = 1u32;
    let mut truncated = false;
    for line in text.lines() {
        if lines.len() >= MAX_DIFF_LINES {
            truncated = true;
            break;
        }
        lines.push(GitDiffLine {
            kind: "add".to_string(),
            content: line.to_string(),
            old_line: 0,
            new_line: new_ln,
        });
        new_ln += 1;
    }
    Ok(GitDiff {
        path: file.to_string(),
        lines,
        is_binary: false,
        truncated,
    })
}

/// Diff của một file trong một commit cụ thể.
pub fn commit_file_diff(repo_path: &str, hash: &str, file: &str) -> AppResult<GitDiff> {
    let raw = run(
        repo_path,
        &["show", "--no-color", "--format=", hash, "--", file],
    )?;
    Ok(parse_diff(file, &raw))
}

// === Staging ===

/// Đưa các file vào staged (`git add`). Nếu `files` rỗng → thêm tất cả (`git add -A`).
pub fn stage(repo_path: &str, files: &[String]) -> AppResult<()> {
    if files.is_empty() {
        run(repo_path, &["add", "-A"])?;
        return Ok(());
    }
    let mut args: Vec<&str> = vec!["add", "--"];
    for f in files {
        args.push(f);
    }
    run(repo_path, &args)?;
    Ok(())
}

/// Bỏ các file khỏi staged (`git restore --staged`). Rỗng → bỏ tất cả.
pub fn unstage(repo_path: &str, files: &[String]) -> AppResult<()> {
    let mut args: Vec<&str> = vec!["restore", "--staged", "--"];
    if files.is_empty() {
        args = vec!["reset", "-q", "HEAD", "--"];
    } else {
        for f in files {
            args.push(f);
        }
    }
    run(repo_path, &args)?;
    Ok(())
}

/// Bỏ thay đổi (discard) của các file. File tracked → `git checkout --`,
/// file untracked → xóa hẳn file khỏi đĩa. `files` không được rỗng.
pub fn discard(repo_path: &str, files: &[String]) -> AppResult<()> {
    for f in files {
        // Bỏ staged trước (nếu có) rồi khôi phục working tree.
        let _ = run(repo_path, &["reset", "-q", "HEAD", "--", f]);
        let restored = run(repo_path, &["checkout", "--", f]);
        if restored.is_err() {
            // File untracked (không có trong HEAD) → xóa khỏi đĩa.
            let abs = Path::new(repo_path).join(f);
            if abs.is_dir() {
                let _ = std::fs::remove_dir_all(&abs);
            } else if abs.exists() {
                let _ = std::fs::remove_file(&abs);
            }
        }
    }
    Ok(())
}

// === Commit ===

/// Tạo commit với message cho trước. Yêu cầu có file staged.
pub fn commit(repo_path: &str, message: &str) -> AppResult<String> {
    if message.trim().is_empty() {
        return Err(AppError::new("Commit message không được để trống."));
    }
    run(repo_path, &["commit", "-m", message])
}

/// Hoàn tác commit gần nhất, giữ lại thay đổi ở staged (`reset --soft HEAD~1`).
pub fn undo_last_commit(repo_path: &str) -> AppResult<String> {
    run(repo_path, &["reset", "--soft", "HEAD~1"])
}

/// Reset branch hiện tại về một commit. `mode`: "soft" | "mixed" | "hard".
pub fn reset_to(repo_path: &str, hash: &str, mode: &str) -> AppResult<String> {
    if hash.trim().is_empty() {
        return Err(AppError::new("Thiếu mã commit để reset."));
    }
    let flag = match mode {
        "soft" => "--soft",
        "hard" => "--hard",
        _ => "--mixed",
    };
    run(repo_path, &["reset", flag, hash])
}

// === Log / History ===

/// Lấy lịch sử commit của branch hiện tại (giới hạn `limit`).
pub fn log(repo_path: &str, limit: u32) -> AppResult<Vec<GitCommit>> {
    log_range(repo_path, "", limit)
}

/// Lấy lịch sử commit. `range` rỗng → HEAD; ngược lại dùng revision range (vd. `base..head`).
pub fn log_range(repo_path: &str, range: &str, limit: u32) -> AppResult<Vec<GitCommit>> {
    let format = format!("--pretty=format:%H{FS}%h{FS}%s{FS}%an{FS}%ae{FS}%aI{FS}%ar{RS}");
    let limit_arg = format!("-{limit}");
    let mut args: Vec<&str> = vec!["log", &limit_arg, &format];
    if !range.trim().is_empty() {
        args.push(range);
    }
    let raw = run(repo_path, &args)?;
    Ok(parse_commits(&raw))
}

/// Parse output `--name-status -z` (kèm rename `-M`) thành danh sách file thay đổi.
fn parse_name_status_z(raw: &str) -> Vec<GitFileChange> {
    let tokens: Vec<&str> = raw.split('\0').filter(|s| !s.is_empty()).collect();
    let mut files: Vec<GitFileChange> = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let code = tokens[i].chars().next().unwrap_or(' ');
        i += 1;
        if code == 'R' || code == 'C' {
            let orig = tokens.get(i).map(|s| s.to_string()).unwrap_or_default();
            let newp = tokens.get(i + 1).map(|s| s.to_string()).unwrap_or_default();
            i += 2;
            files.push(GitFileChange {
                path: newp,
                orig_path: orig,
                status: code.to_string(),
                untracked: false,
            });
        } else {
            let p = tokens.get(i).map(|s| s.to_string()).unwrap_or_default();
            i += 1;
            files.push(GitFileChange {
                path: p,
                orig_path: String::new(),
                status: code.to_string(),
                untracked: false,
            });
        }
    }
    files
}

fn parse_commits(raw: &str) -> Vec<GitCommit> {
    raw.split(RS)
        .map(|r| r.trim_matches(['\n', '\r']))
        .filter(|r| !r.is_empty())
        .filter_map(|record| {
            let f: Vec<&str> = record.split(FS).collect();
            if f.len() < 7 {
                return None;
            }
            Some(GitCommit {
                hash: f[0].to_string(),
                short_hash: f[1].to_string(),
                subject: f[2].to_string(),
                author_name: f[3].to_string(),
                author_email: f[4].to_string(),
                date: f[5].to_string(),
                relative_date: f[6].to_string(),
            })
        })
        .collect()
}

/// Chi tiết một commit: meta + body + danh sách file đã đổi.
pub fn commit_detail(repo_path: &str, hash: &str) -> AppResult<GitCommitDetail> {
    let format = format!("--pretty=format:%H{FS}%h{FS}%s{FS}%an{FS}%ae{FS}%aI{FS}%ar{FS}%b");
    let raw = run(repo_path, &["show", "-s", &format, hash])?;
    let f: Vec<&str> = raw.split(FS).collect();
    if f.len() < 7 {
        return Err(AppError::new("Không đọc được thông tin commit."));
    }
    let commit = GitCommit {
        hash: f[0].to_string(),
        short_hash: f[1].to_string(),
        subject: f[2].to_string(),
        author_name: f[3].to_string(),
        author_email: f[4].to_string(),
        date: f[5].to_string(),
        relative_date: f[6].to_string(),
    };
    let body = f.get(7).map(|s| s.trim().to_string()).unwrap_or_default();

    // Danh sách file đã đổi trong commit.
    let names = run(
        repo_path,
        &[
            "diff-tree",
            "--no-commit-id",
            "--name-status",
            "-M",
            "-r",
            "-z",
            hash,
        ],
    )?;
    let files = parse_name_status_z(&names);

    Ok(GitCommitDetail {
        commit,
        body,
        files,
    })
}

// === Branches ===

/// Liệt kê tất cả branch (local + remote).
pub fn branches(repo_path: &str) -> AppResult<Vec<GitBranch>> {
    let format = format!(
        "--format=%(refname){FS}%(HEAD){FS}%(refname:short){FS}%(upstream:short){FS}%(contents:subject)"
    );
    let raw = run(
        repo_path,
        &["for-each-ref", &format, "refs/heads", "refs/remotes"],
    )?;

    let mut out: Vec<GitBranch> = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split(FS).collect();
        if f.len() < 5 {
            continue;
        }
        let full = f[0];
        let short = f[2].to_string();
        // Bỏ qua con trỏ symbolic origin/HEAD.
        if short.ends_with("/HEAD") {
            continue;
        }
        let is_remote = full.starts_with("refs/remotes/");
        out.push(GitBranch {
            name: short,
            is_current: f[1] == "*",
            is_remote,
            upstream: f[3].to_string(),
            last_commit_subject: f[4].to_string(),
        });
    }
    Ok(out)
}

/// Checkout sang một branch đã tồn tại.
pub fn checkout_branch(repo_path: &str, name: &str) -> AppResult<String> {
    run(repo_path, &["checkout", name])
}

/// Tạo branch mới từ HEAD (hoặc từ `from` nếu có) và checkout sang đó.
pub fn create_branch(repo_path: &str, name: &str, from: &str) -> AppResult<String> {
    if name.trim().is_empty() {
        return Err(AppError::new("Tên branch không được để trống."));
    }
    if from.trim().is_empty() {
        run(repo_path, &["checkout", "-b", name])
    } else {
        run(repo_path, &["checkout", "-b", name, from])
    }
}

/// Xóa một branch local. `force = true` → dùng `-D`.
pub fn delete_branch(repo_path: &str, name: &str, force: bool) -> AppResult<String> {
    let flag = if force { "-D" } else { "-d" };
    run(repo_path, &["branch", flag, name])
}

// === Revert ===

/// Revert một commit (tạo commit mới đảo ngược thay đổi). Dùng `--no-edit`.
pub fn revert(repo_path: &str, hash: &str) -> AppResult<String> {
    if hash.trim().is_empty() {
        return Err(AppError::new("Thiếu mã commit để revert."));
    }
    run(repo_path, &["revert", "--no-edit", hash])
}

/// Hủy một revert đang dở (khi có xung đột).
pub fn revert_abort(repo_path: &str) -> AppResult<String> {
    run(repo_path, &["revert", "--abort"])
}

// === Rebase ===

/// Rebase branch hiện tại lên trên `onto` (branch/ref đích).
pub fn rebase(repo_path: &str, onto: &str) -> AppResult<String> {
    if onto.trim().is_empty() {
        return Err(AppError::new("Thiếu branch đích để rebase."));
    }
    run(repo_path, &["rebase", onto])
}

/// Hủy rebase đang dở, đưa branch về trạng thái trước rebase.
pub fn rebase_abort(repo_path: &str) -> AppResult<String> {
    run(repo_path, &["rebase", "--abort"])
}

/// Chạy một lệnh git với editor không tương tác (`GIT_EDITOR=true`) — dùng cho
/// các lệnh `--continue` vốn sẽ mở editor commit message và làm treo app.
fn run_no_editor(repo_path: &str, args: &[&str]) -> AppResult<String> {
    let mut cmd = git_in(repo_path);
    cmd.env("GIT_EDITOR", "true");
    let output = cmd
        .args(args)
        .output()
        .map_err(|e| AppError::new(format!("Không chạy được git: {e}")))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(AppError::new(if stderr.is_empty() {
            "Lệnh git thất bại.".to_string()
        } else {
            stderr
        }))
    }
}

/// Tiếp tục rebase sau khi đã giải quyết xung đột (và stage các file).
pub fn rebase_continue(repo_path: &str) -> AppResult<String> {
    run_no_editor(repo_path, &["rebase", "--continue"])
}

// === Cherry-pick ===

/// Cherry-pick một commit vào branch hiện tại.
pub fn cherry_pick(repo_path: &str, hash: &str) -> AppResult<String> {
    if hash.trim().is_empty() {
        return Err(AppError::new("Thiếu mã commit để cherry-pick."));
    }
    run(repo_path, &["cherry-pick", hash])
}

/// Hủy cherry-pick đang dở (khi có xung đột).
pub fn cherry_pick_abort(repo_path: &str) -> AppResult<String> {
    run(repo_path, &["cherry-pick", "--abort"])
}

/// Tiếp tục cherry-pick sau khi đã giải quyết xung đột (và stage các file).
pub fn cherry_pick_continue(repo_path: &str) -> AppResult<String> {
    run_no_editor(repo_path, &["cherry-pick", "--continue"])
}

// === Thao tác mạng có tiến trình (fetch/pull/push/clone) ===

/// Parse một dòng progress của git (vd. "Receiving objects:  45% (450/1000)").
fn parse_progress(line: &str) -> Option<GitProgress> {
    let l = line.trim();
    let l = l.strip_prefix("remote: ").unwrap_or(l).trim();
    let pct_pos = l.find('%')?;
    let bytes = l.as_bytes();
    let mut start = pct_pos;
    while start > 0 && bytes[start - 1].is_ascii_digit() {
        start -= 1;
    }
    if start == pct_pos {
        return None;
    }
    let percent: u32 = l[start..pct_pos].parse().ok()?;
    let phase = l.split(':').next().unwrap_or("").trim().to_string();
    Some(GitProgress {
        phase,
        percent: percent.min(100),
        raw: l.to_string(),
    })
}

/// Chạy một lệnh git (đã cấu hình), stream stderr để bắt tiến trình `--progress`,
/// gọi `on` cho mỗi mốc %. Trả về stdout khi thành công.
fn run_progress<F: FnMut(GitProgress)>(mut cmd: Command, mut on: F) -> AppResult<String> {
    use std::io::Read;
    use std::process::Stdio;

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd
        .spawn()
        .map_err(|e| AppError::new(format!("Không chạy được git: {e}. Hãy chắc chắn đã cài Git.")))?;

    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::new("Không đọc được tiến trình git."))?;

    let mut buf = [0u8; 4096];
    let mut line = String::new();
    let mut all_err = String::new();
    loop {
        match stderr.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buf[..n]);
                all_err.push_str(&chunk);
                // Git dùng '\r' để cập nhật cùng một dòng progress.
                for ch in chunk.chars() {
                    if ch == '\r' || ch == '\n' {
                        if !line.trim().is_empty() {
                            if let Some(p) = parse_progress(&line) {
                                on(p);
                            }
                        }
                        line.clear();
                    } else {
                        line.push(ch);
                    }
                }
            }
            Err(_) => break,
        }
    }
    if !line.trim().is_empty() {
        if let Some(p) = parse_progress(&line) {
            on(p);
        }
    }

    let mut out = String::new();
    if let Some(mut so) = child.stdout.take() {
        let _ = so.read_to_string(&mut out);
    }
    let status = child
        .wait()
        .map_err(|e| AppError::new(format!("Lỗi chờ tiến trình git: {e}")))?;

    if status.success() {
        Ok(out)
    } else {
        // Bỏ các dòng progress (%) khi dựng thông báo lỗi.
        let clean: Vec<&str> = all_err
            .split(['\r', '\n'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.contains('%'))
            .collect();
        let msg = if clean.is_empty() {
            all_err.trim().to_string()
        } else {
            clean.join("\n")
        };
        Err(AppError::new(if msg.is_empty() {
            "Lệnh git thất bại.".to_string()
        } else {
            msg
        }))
    }
}

/// Fetch có tiến trình.
pub fn fetch_with_progress<F: FnMut(GitProgress)>(repo_path: &str, on: F) -> AppResult<String> {
    let mut cmd = git_in(repo_path);
    cmd.args(["fetch", "--all", "--prune", "--progress"]);
    run_progress(cmd, on)
}

/// Pull có tiến trình.
pub fn pull_with_progress<F: FnMut(GitProgress)>(repo_path: &str, on: F) -> AppResult<String> {
    let mut cmd = git_in(repo_path);
    cmd.args(["pull", "--progress"]);
    run_progress(cmd, on)
}

/// Push có tiến trình. Chưa có upstream → tự set `-u origin <branch>`.
pub fn push_with_progress<F: FnMut(GitProgress)>(repo_path: &str, on: F) -> AppResult<String> {
    let info = repo_info(repo_path)?;
    let mut cmd = git_in(repo_path);
    if info.upstream.is_empty() {
        if info.current_branch.is_empty() || info.detached {
            return Err(AppError::new(
                "Đang ở detached HEAD — hãy checkout một branch trước khi push.",
            ));
        }
        cmd.args(["push", "--progress", "-u", "origin", &info.current_branch]);
    } else {
        cmd.args(["push", "--progress"]);
    }
    run_progress(cmd, on)
}

/// Clone có tiến trình.
pub fn clone_with_progress<F: FnMut(GitProgress)>(
    url: &str,
    dest: &str,
    on: F,
) -> AppResult<String> {
    if url.trim().is_empty() {
        return Err(AppError::new("URL repository không được để trống."));
    }
    let mut cmd = Command::new("git");
    configure(&mut cmd);
    cmd.args(["clone", "--progress", url, dest]);
    run_progress(cmd, on)?;
    Ok(dest.to_string())
}

// === Tag ===

/// Liệt kê tag (mới nhất trước).
pub fn tag_list(repo_path: &str) -> AppResult<Vec<GitTag>> {
    let format = format!(
        "--format=%(refname:short){FS}%(objectname:short){FS}%(contents:subject){FS}%(creatordate:short)"
    );
    let raw = run(
        repo_path,
        &["for-each-ref", "--sort=-creatordate", &format, "refs/tags"],
    )?;
    let mut out = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split(FS).collect();
        if f.is_empty() {
            continue;
        }
        out.push(GitTag {
            name: f[0].to_string(),
            target: f.get(1).map(|s| s.to_string()).unwrap_or_default(),
            subject: f.get(2).map(|s| s.to_string()).unwrap_or_default(),
            date: f.get(3).map(|s| s.to_string()).unwrap_or_default(),
        });
    }
    Ok(out)
}

/// Tạo tag. `annotated` → tag có message (`-a -m`); `hash` rỗng → dùng HEAD.
/// `push` → đẩy tag lên origin sau khi tạo.
pub fn tag_create(
    repo_path: &str,
    name: &str,
    hash: &str,
    message: &str,
    annotated: bool,
    push: bool,
) -> AppResult<String> {
    if name.trim().is_empty() {
        return Err(AppError::new("Tên tag không được để trống."));
    }
    // Annotated cần message (rỗng sẽ mở editor và treo) — fallback dùng tên tag.
    let msg = if message.trim().is_empty() { name } else { message };
    let mut args: Vec<&str> = vec!["tag"];
    if annotated {
        args.push("-a");
        args.push(name);
        args.push("-m");
        args.push(msg);
    } else {
        args.push(name);
    }
    if !hash.trim().is_empty() {
        args.push(hash);
    }
    run(repo_path, &args)?;
    if push {
        run(repo_path, &["push", "origin", name])?;
    }
    Ok(name.to_string())
}

/// Xóa một tag local. `remote = true` → xóa cả trên origin.
pub fn tag_delete(repo_path: &str, name: &str, remote: bool) -> AppResult<String> {
    run(repo_path, &["tag", "-d", name])?;
    if remote {
        run(repo_path, &["push", "origin", "--delete", name])?;
    }
    Ok(name.to_string())
}

// === Merge ===

/// Merge một branch vào branch hiện tại.
/// - `squash = true` → `merge --squash` rồi commit (một commit gộp).
/// - ngược lại → merge thường (tạo merge commit hoặc fast-forward).
pub fn merge(repo_path: &str, branch: &str, squash: bool, message: &str) -> AppResult<String> {
    if branch.trim().is_empty() {
        return Err(AppError::new("Thiếu branch để merge."));
    }
    if squash {
        run(repo_path, &["merge", "--squash", branch])?;
        let default_msg = format!("Squash merge branch '{branch}'");
        let msg = if message.trim().is_empty() {
            default_msg.as_str()
        } else {
            message
        };
        run(repo_path, &["commit", "-m", msg])
    } else {
        run(repo_path, &["merge", "--no-edit", branch])
    }
}

/// Hủy merge đang dở (khi có xung đột).
pub fn merge_abort(repo_path: &str) -> AppResult<String> {
    run(repo_path, &["merge", "--abort"])
}

/// Commit để hoàn tất merge (giữ message mặc định, không mở editor).
pub fn commit_no_edit(repo_path: &str) -> AppResult<String> {
    run_no_editor(repo_path, &["commit", "--no-edit"])
}

// === Resolve conflict ===

/// Danh sách file đang xung đột (unmerged).
pub fn list_conflicts(repo_path: &str) -> AppResult<Vec<String>> {
    let raw = run(
        repo_path,
        &["diff", "--name-only", "--diff-filter=U", "-z"],
    )?;
    Ok(raw
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect())
}

/// Giải quyết xung đột một file bằng cách chọn một phía rồi stage.
/// `side`: "ours" (bản HEAD) | "theirs" (bản đến).
pub fn resolve_conflict(repo_path: &str, file: &str, side: &str) -> AppResult<()> {
    let flag = if side == "theirs" { "--theirs" } else { "--ours" };
    run(repo_path, &["checkout", flag, "--", file])?;
    run(repo_path, &["add", "--", file])?;
    Ok(())
}

// === Cleanup branch đã merge (upstream bị xóa) ===

/// Fetch --prune rồi trả về các branch local có upstream đã bị xóa ([gone]),
/// trừ branch hiện tại. Dùng để dọn branch sau khi PR đã merge + xóa remote.
pub fn cleanup_scan(repo_path: &str) -> AppResult<Vec<String>> {
    // Prune remote-tracking refs trước để phát hiện upstream đã mất.
    let _ = run(repo_path, &["fetch", "--all", "--prune"]);

    let current = run(repo_path, &["symbolic-ref", "--short", "-q", "HEAD"])
        .unwrap_or_default()
        .trim()
        .to_string();

    let fmt = format!("--format=%(refname:short){FS}%(upstream:track)");
    let raw = run(repo_path, &["for-each-ref", &fmt, "refs/heads"])?;

    let mut out = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split(FS).collect();
        let name = f[0].trim();
        let track = f.get(1).copied().unwrap_or("");
        if track.contains("gone") && !name.is_empty() && name != current {
            out.push(name.to_string());
        }
    }
    Ok(out)
}

/// Xóa (force) các branch local đã chọn. Trả về danh sách đã xóa thành công.
pub fn cleanup_delete(repo_path: &str, branches: &[String]) -> AppResult<Vec<String>> {
    let mut deleted = Vec::new();
    for b in branches {
        if run(repo_path, &["branch", "-D", b]).is_ok() {
            deleted.push(b.clone());
        }
    }
    Ok(deleted)
}

// === Compare / Pull Request ===

fn count_range(repo_path: &str, range: &str) -> u32 {
    run_opt(repo_path, &["rev-list", "--count", range])
        .parse()
        .unwrap_or(0)
}

/// So sánh 2 branch: commit `base..head` + file `base...head` + URL tạo PR.
pub fn compare(repo_path: &str, base: &str, head: &str) -> AppResult<GitComparison> {
    if base.trim().is_empty() || head.trim().is_empty() {
        return Err(AppError::new("Cần chọn cả base và head để so sánh."));
    }
    let ahead = count_range(repo_path, &format!("{base}..{head}"));
    let behind = count_range(repo_path, &format!("{head}..{base}"));
    let commits = log_range(repo_path, &format!("{base}..{head}"), 300)?;

    let names = run(
        repo_path,
        &[
            "diff",
            "--name-status",
            "-M",
            "-z",
            &format!("{base}...{head}"),
        ],
    )?;
    let files = parse_name_status_z(&names);

    let web_url = remote_web_url(repo_path);
    let pr_url = pull_request_url(&web_url, base, head);

    Ok(GitComparison {
        base: base.to_string(),
        head: head.to_string(),
        ahead,
        behind,
        commits,
        files,
        web_url,
        pr_url,
    })
}

/// Diff của một file giữa 2 branch (`base...head`).
pub fn compare_file_diff(
    repo_path: &str,
    base: &str,
    head: &str,
    file: &str,
) -> AppResult<GitDiff> {
    let raw = run(
        repo_path,
        &["diff", "--no-color", &format!("{base}...{head}"), "--", file],
    )?;
    Ok(parse_diff(file, &raw))
}

/// Chuyển remote origin (ssh/https) thành URL web `https://host/owner/repo`.
fn remote_web_url(repo_path: &str) -> String {
    let raw = run_opt(repo_path, &["remote", "get-url", "origin"]);
    if raw.is_empty() {
        return String::new();
    }
    let mut u = raw.trim().to_string();
    if let Some(rest) = u.strip_prefix("git@") {
        // git@host:owner/repo(.git)
        if let Some((host, path)) = rest.split_once(':') {
            u = format!("https://{host}/{path}");
        }
    } else if let Some(rest) = u.strip_prefix("ssh://") {
        let rest = rest.strip_prefix("git@").unwrap_or(rest);
        u = format!("https://{rest}");
    }
    if let Some(stripped) = u.strip_suffix(".git") {
        u = stripped.to_string();
    }
    while u.ends_with('/') {
        u.pop();
    }
    u
}

/// Dựng URL tạo Pull Request/Merge Request theo host (GitHub/GitLab/Bitbucket).
fn pull_request_url(web: &str, base: &str, head: &str) -> String {
    if web.is_empty() {
        return String::new();
    }
    if web.contains("gitlab") {
        let h = urlencoding::encode(head);
        let b = urlencoding::encode(base);
        format!(
            "{web}/-/merge_requests/new?merge_request[source_branch]={h}&merge_request[target_branch]={b}"
        )
    } else if web.contains("bitbucket") {
        let h = urlencoding::encode(head);
        let b = urlencoding::encode(base);
        format!("{web}/pull-requests/new?source={h}&dest={b}")
    } else {
        // GitHub-style compare (giữ nguyên dấu `/` trong tên branch cho hợp lệ).
        format!("{web}/compare/{base}...{head}?expand=1")
    }
}

/// Mở một URL bằng trình duyệt mặc định của hệ điều hành.
pub fn open_url(url: &str) -> AppResult<()> {
    if url.trim().is_empty() {
        return Err(AppError::new("URL rỗng."));
    }
    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("cmd");
        configure(&mut cmd);
        cmd.args(["/C", "start", "", url])
            .spawn()
            .map_err(|e| AppError::new(format!("Không mở được trình duyệt: {e}")))?;
    }
    #[cfg(target_os = "macos")]
    {
        let mut cmd = Command::new("open");
        configure(&mut cmd);
        cmd.arg(url)
            .spawn()
            .map_err(|e| AppError::new(format!("Không mở được trình duyệt: {e}")))?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let mut cmd = Command::new("xdg-open");
        configure(&mut cmd);
        cmd.arg(url)
            .spawn()
            .map_err(|e| AppError::new(format!("Không mở được trình duyệt: {e}")))?;
    }
    Ok(())
}

/// Tạo Pull Request: mở trang tạo PR trên host (GitHub Desktop cũng mở trình duyệt).
/// Trả về URL đã mở.
pub fn create_pull_request(repo_path: &str, base: &str, head: &str) -> AppResult<String> {
    let web = remote_web_url(repo_path);
    if web.is_empty() {
        return Err(AppError::new(
            "Không tìm thấy remote origin để tạo Pull Request.",
        ));
    }
    let url = pull_request_url(&web, base, head);
    open_url(&url)?;
    Ok(url)
}

// === Pull Request list (gọi API host) ===

/// Thông tin host để gọi API liệt kê Pull Request.
pub struct PrSource {
    /// "github" | "gitlab" | "bitbucket" | "other".
    pub kind: String,
    pub host: String,
    pub owner: String,
    pub repo: String,
}

/// Phân tích host/owner/repo từ remote origin.
pub fn pr_source(repo_path: &str) -> AppResult<PrSource> {
    let web = remote_web_url(repo_path);
    if web.is_empty() {
        return Err(AppError::new("Repo không có remote origin."));
    }
    let after = web
        .strip_prefix("https://")
        .or_else(|| web.strip_prefix("http://"))
        .unwrap_or(&web);
    let (host, path) = after
        .split_once('/')
        .ok_or_else(|| AppError::new("Không phân tích được URL remote."))?;
    let segs: Vec<&str> = path.trim_matches('/').split('/').filter(|s| !s.is_empty()).collect();
    if segs.len() < 2 {
        return Err(AppError::new("Không xác định được owner/repo từ remote."));
    }
    let repo = segs[segs.len() - 1].to_string();
    let owner = segs[..segs.len() - 1].join("/");
    let kind = if host.contains("github") {
        "github"
    } else if host.contains("gitlab") {
        "gitlab"
    } else if host.contains("bitbucket") {
        "bitbucket"
    } else {
        "other"
    }
    .to_string();
    Ok(PrSource {
        kind,
        host: host.to_string(),
        owner,
        repo,
    })
}

/// Lấy token cho `host` từ git credential helper (best-effort; rỗng nếu không có).
/// Tận dụng credential đã lưu để không cần cấu hình token riêng trong app.
pub fn credential_token(host: &str) -> String {
    use std::io::Write;
    use std::process::Stdio;

    let mut cmd = Command::new("git");
    configure(&mut cmd);
    let child = cmd
        .args(["credential", "fill"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();
    let mut child = match child {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    if let Some(mut sin) = child.stdin.take() {
        let _ = write!(sin, "protocol=https\nhost={host}\n\n");
    }
    let out = match child.wait_with_output() {
        Ok(o) => o,
        Err(_) => return String::new(),
    };
    if !out.status.success() {
        return String::new();
    }
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        if let Some(p) = line.strip_prefix("password=") {
            return p.trim().to_string();
        }
    }
    String::new()
}

fn build_http() -> AppResult<Client> {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::new(format!("Không tạo được HTTP client: {e}")))
}

/// Liệt kê Pull Request / Merge Request. `state`: "open" | "closed" | "all".
pub async fn list_pull_requests(repo_path: &str, state: &str) -> AppResult<Vec<GitPullRequest>> {
    let src = pr_source(repo_path)?;
    let token = credential_token(&src.host);
    match src.kind.as_str() {
        "github" => github_pull_requests(&src, &token, state).await,
        "gitlab" => gitlab_merge_requests(&src, &token, state).await,
        _ => Err(AppError::new(
            "Chỉ hỗ trợ liệt kê Pull Request cho GitHub và GitLab.",
        )),
    }
}

async fn github_pull_requests(
    src: &PrSource,
    token: &str,
    state: &str,
) -> AppResult<Vec<GitPullRequest>> {
    let api = if src.host == "github.com" {
        "https://api.github.com".to_string()
    } else {
        format!("https://{}/api/v3", src.host)
    };
    let gh_state = match state {
        "closed" => "closed",
        "all" => "all",
        _ => "open",
    };
    let url = format!(
        "{api}/repos/{}/{}/pulls?state={gh_state}&per_page=50&sort=updated&direction=desc",
        src.owner, src.repo
    );
    let client = build_http()?;
    let mut req = client
        .get(&url)
        .header("User-Agent", "management-systems")
        .header("Accept", "application/vnd.github+json");
    if !token.is_empty() {
        req = req.header("Authorization", format!("Bearer {token}"));
    }
    let resp = req
        .send()
        .await
        .map_err(|e| AppError::new(format!("Gọi GitHub API lỗi: {e}")))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        let msg = serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|v| v["message"].as_str().map(|s| s.to_string()))
            .unwrap_or(body);
        return Err(AppError::new(format!(
            "GitHub API {}: {msg}",
            status.as_u16()
        )));
    }
    let arr: Vec<serde_json::Value> = resp
        .json()
        .await
        .map_err(|e| AppError::new(format!("Lỗi đọc dữ liệu GitHub: {e}")))?;
    let out = arr
        .iter()
        .map(|v| {
            let draft = v["draft"].as_bool().unwrap_or(false);
            let st = v["state"].as_str().unwrap_or("open").to_string();
            GitPullRequest {
                number: v["number"].as_u64().unwrap_or(0),
                title: v["title"].as_str().unwrap_or("").to_string(),
                author: v["user"]["login"].as_str().unwrap_or("").to_string(),
                state: if draft && st == "open" { "draft".to_string() } else { st },
                draft,
                head: v["head"]["ref"].as_str().unwrap_or("").to_string(),
                base: v["base"]["ref"].as_str().unwrap_or("").to_string(),
                url: v["html_url"].as_str().unwrap_or("").to_string(),
                created_at: v["created_at"].as_str().unwrap_or("").to_string(),
                updated_at: v["updated_at"].as_str().unwrap_or("").to_string(),
            }
        })
        .collect();
    Ok(out)
}

async fn gitlab_merge_requests(
    src: &PrSource,
    token: &str,
    state: &str,
) -> AppResult<Vec<GitPullRequest>> {
    let proj = urlencoding::encode(&format!("{}/{}", src.owner, src.repo)).into_owned();
    let gl_state = match state {
        "closed" => "closed",
        "all" => "all",
        _ => "opened",
    };
    let url = format!(
        "https://{}/api/v4/projects/{}/merge_requests?state={gl_state}&per_page=50&order_by=updated_at",
        src.host, proj
    );
    let client = build_http()?;
    let mut req = client.get(&url).header("User-Agent", "management-systems");
    if !token.is_empty() {
        req = req.header("PRIVATE-TOKEN", token);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| AppError::new(format!("Gọi GitLab API lỗi: {e}")))?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::new(format!("GitLab API {}: {body}", status.as_u16())));
    }
    let arr: Vec<serde_json::Value> = resp
        .json()
        .await
        .map_err(|e| AppError::new(format!("Lỗi đọc dữ liệu GitLab: {e}")))?;
    let out = arr
        .iter()
        .map(|v| {
            let draft = v["draft"].as_bool().or_else(|| v["work_in_progress"].as_bool()).unwrap_or(false);
            let raw_state = v["state"].as_str().unwrap_or("opened");
            let st = match raw_state {
                "opened" => if draft { "draft" } else { "open" },
                other => other,
            }
            .to_string();
            GitPullRequest {
                number: v["iid"].as_u64().unwrap_or(0),
                title: v["title"].as_str().unwrap_or("").to_string(),
                author: v["author"]["username"].as_str().unwrap_or("").to_string(),
                state: st,
                draft,
                head: v["source_branch"].as_str().unwrap_or("").to_string(),
                base: v["target_branch"].as_str().unwrap_or("").to_string(),
                url: v["web_url"].as_str().unwrap_or("").to_string(),
                created_at: v["created_at"].as_str().unwrap_or("").to_string(),
                updated_at: v["updated_at"].as_str().unwrap_or("").to_string(),
            }
        })
        .collect();
    Ok(out)
}

// === Worktree ===

/// Liệt kê các worktree của repo.
pub fn worktree_list(repo_path: &str) -> AppResult<Vec<GitWorktree>> {
    let raw = run(repo_path, &["worktree", "list", "--porcelain"])?;
    let top = normalize_path(&run_opt(repo_path, &["rev-parse", "--show-toplevel"]));

    let mut out: Vec<GitWorktree> = Vec::new();
    let mut cur: Option<GitWorktree> = None;
    for line in raw.lines() {
        if line.trim().is_empty() {
            if let Some(w) = cur.take() {
                out.push(w);
            }
            continue;
        }
        if let Some(p) = line.strip_prefix("worktree ") {
            if let Some(w) = cur.take() {
                out.push(w);
            }
            cur = Some(GitWorktree {
                path: p.to_string(),
                head: String::new(),
                branch: String::new(),
                is_bare: false,
                is_detached: false,
                is_current: false,
            });
        } else if let Some(w) = cur.as_mut() {
            if let Some(h) = line.strip_prefix("HEAD ") {
                w.head = h.to_string();
            } else if let Some(b) = line.strip_prefix("branch ") {
                w.branch = b.trim_start_matches("refs/heads/").to_string();
            } else if line == "bare" {
                w.is_bare = true;
            } else if line == "detached" {
                w.is_detached = true;
            }
        }
    }
    if let Some(w) = cur.take() {
        out.push(w);
    }

    for w in out.iter_mut() {
        w.is_current = normalize_path(&w.path) == top && !top.is_empty();
    }
    Ok(out)
}

/// Chuẩn hóa đường dẫn để so sánh (đồng nhất dấu phân tách, bỏ `/` cuối).
fn normalize_path(p: &str) -> String {
    let mut s = p.replace('\\', "/");
    while s.len() > 1 && s.ends_with('/') {
        s.pop();
    }
    #[cfg(target_os = "windows")]
    {
        s = s.to_lowercase();
    }
    s
}

/// Tạo worktree mới tại `path`.
/// - `new_branch` không rỗng → tạo branch mới (`-b`), xuất phát từ `branch` nếu có.
/// - ngược lại → checkout `branch` (đã tồn tại) vào worktree; rỗng thì dùng HEAD.
pub fn worktree_add(
    repo_path: &str,
    path: &str,
    branch: &str,
    new_branch: &str,
) -> AppResult<String> {
    if path.trim().is_empty() {
        return Err(AppError::new("Thiếu đường dẫn cho worktree."));
    }
    let mut args: Vec<&str> = vec!["worktree", "add"];
    if !new_branch.trim().is_empty() {
        args.push("-b");
        args.push(new_branch);
        args.push(path);
        if !branch.trim().is_empty() {
            args.push(branch);
        }
    } else {
        args.push(path);
        if !branch.trim().is_empty() {
            args.push(branch);
        }
    }
    run(repo_path, &args)?;
    Ok(path.to_string())
}

/// Gỡ một worktree. `force = true` → dùng `--force`.
pub fn worktree_remove(repo_path: &str, path: &str, force: bool) -> AppResult<String> {
    let mut args: Vec<&str> = vec!["worktree", "remove"];
    if force {
        args.push("--force");
    }
    args.push(path);
    run(repo_path, &args)
}

// === Network (fetch / pull / push) ===

/// Fetch tất cả remote + prune branch đã xóa.
pub fn fetch(repo_path: &str) -> AppResult<String> {
    run(repo_path, &["fetch", "--all", "--prune"])
}

/// Pull branch hiện tại từ upstream.
pub fn pull(repo_path: &str) -> AppResult<String> {
    run(repo_path, &["pull"])
}

/// Push branch hiện tại. Nếu chưa có upstream → tự set `-u origin <branch>`.
pub fn push(repo_path: &str) -> AppResult<String> {
    let info = repo_info(repo_path)?;
    if info.upstream.is_empty() {
        if info.current_branch.is_empty() || info.detached {
            return Err(AppError::new(
                "Đang ở detached HEAD — hãy checkout một branch trước khi push.",
            ));
        }
        run(
            repo_path,
            &["push", "-u", "origin", &info.current_branch],
        )
    } else {
        run(repo_path, &["push"])
    }
}

// === Stash ===

/// Liệt kê stash.
pub fn stash_list(repo_path: &str) -> AppResult<Vec<GitStash>> {
    let format = format!("--pretty=format:%gd{FS}%s");
    let raw = run(repo_path, &["stash", "list", &format])?;
    let mut out = Vec::new();
    for (idx, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split(FS).collect();
        let reference = f.first().map(|s| s.to_string()).unwrap_or_default();
        let message = f.get(1).map(|s| s.to_string()).unwrap_or_default();
        out.push(GitStash {
            index: idx as u32,
            reference,
            message,
        });
    }
    Ok(out)
}

/// Cất thay đổi hiện tại vào stash (kèm untracked). `message` có thể rỗng.
pub fn stash_save(repo_path: &str, message: &str) -> AppResult<String> {
    let mut args: Vec<&str> = vec!["stash", "push", "--include-untracked"];
    if !message.trim().is_empty() {
        args.push("-m");
        args.push(message);
    }
    run(repo_path, &args)
}

/// Áp dụng một stash. `pop = true` → apply rồi xóa stash.
pub fn stash_apply(repo_path: &str, reference: &str, pop: bool) -> AppResult<String> {
    let sub = if pop { "pop" } else { "apply" };
    run(repo_path, &["stash", sub, reference])
}

/// Xóa một stash.
pub fn stash_drop(repo_path: &str, reference: &str) -> AppResult<String> {
    run(repo_path, &["stash", "drop", reference])
}

// === Clone ===

/// Clone một repo về `dest` (thư mục đích đầy đủ). Trả về đường dẫn đích.
pub fn clone(url: &str, dest: &str) -> AppResult<String> {
    if url.trim().is_empty() {
        return Err(AppError::new("URL repository không được để trống."));
    }
    let mut cmd = Command::new("git");
    configure(&mut cmd);
    let output = cmd
        .args(["clone", url, dest])
        .output()
        .map_err(|e| AppError::new(format!("Không chạy được git: {e}")))?;
    if output.status.success() {
        Ok(dest.to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(AppError::new(if stderr.is_empty() {
            "Clone thất bại.".to_string()
        } else {
            stderr
        }))
    }
}
