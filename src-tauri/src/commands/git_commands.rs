//! Tauri command handlers cho màn hình Git Desktop.
//!
//! Gồm 2 nhóm:
//! - Quản lý danh sách repo (lưu cục bộ JSON): list/add/remove/touch.
//! - Thao tác Git trên một repo: status, diff, stage, commit, log, branch,
//!   fetch/pull/push, stash, clone.
//!
//! Các thao tác chạm mạng / ghi nặng chạy qua `spawn_blocking` để không chặn UI.

use crate::app::error::log_err;
use crate::database::git_repo_store::{self, GitRepoData};
use crate::models::git::{
    GitBranch, GitCommit, GitCommitDetail, GitDiff, GitRepo, GitRepoInfo, GitStash, GitStatus,
    GitWorktree,
};
use crate::services::git_service;

// === Quản lý danh sách repo ===

/// Danh sách repo đã lưu, sắp theo lần mở gần nhất giảm dần.
#[tauri::command]
pub fn git_list_repos() -> Result<Vec<GitRepo>, String> {
    let data = git_repo_store::load().map_err(log_err)?;
    let mut repos = data.repos;
    repos.sort_by(|a, b| b.last_opened.cmp(&a.last_opened));
    Ok(repos)
}

/// Thêm một repo vào danh sách. Kiểm tra là git repo hợp lệ và không trùng đường dẫn.
#[tauri::command]
pub fn git_add_repo(path: String) -> Result<GitRepo, String> {
    if !git_service::is_git_repo(&path) {
        return Err(format!("Thư mục không phải Git repository: {path}"));
    }
    let mut data = git_repo_store::load().map_err(log_err)?;

    // Chuẩn hóa: lấy top-level của repo để tránh thêm thư mục con.
    let info = git_service::repo_info(&path).map_err(log_err)?;
    let canonical = info.path;

    if let Some(existing) = data.repos.iter().find(|r| r.path == canonical) {
        return Ok(existing.clone());
    }

    let name = std::path::Path::new(&canonical)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| canonical.clone());

    if data.next_id < 1 {
        data.next_id = 1;
    }
    let repo = GitRepo {
        id: data.next_id,
        name,
        path: canonical,
        last_opened: chrono::Local::now().to_rfc3339(),
    };
    data.next_id += 1;
    data.repos.push(repo.clone());
    git_repo_store::save(&data).map_err(log_err)?;
    Ok(repo)
}

/// Xóa một repo khỏi danh sách (không đụng tới file trên đĩa).
#[tauri::command]
pub fn git_remove_repo(id: i64) -> Result<(), String> {
    let mut data = git_repo_store::load().map_err(log_err)?;
    data.repos.retain(|r| r.id != id);
    git_repo_store::save(&data).map_err(log_err)?;
    Ok(())
}

/// Cập nhật thời điểm mở gần nhất của repo (dùng để sắp xếp recent).
#[tauri::command]
pub fn git_touch_repo(id: i64) -> Result<(), String> {
    let mut data: GitRepoData = git_repo_store::load().map_err(log_err)?;
    if let Some(repo) = data.repos.iter_mut().find(|r| r.id == id) {
        repo.last_opened = chrono::Local::now().to_rfc3339();
        git_repo_store::save(&data).map_err(log_err)?;
    }
    Ok(())
}

// === Đọc trạng thái repo (nhanh, chạy đồng bộ) ===

#[tauri::command]
pub fn git_repo_info(path: String) -> Result<GitRepoInfo, String> {
    git_service::repo_info(&path).map_err(log_err)
}

#[tauri::command]
pub fn git_status(path: String) -> Result<GitStatus, String> {
    git_service::status(&path).map_err(log_err)
}

#[tauri::command]
pub fn git_file_diff(
    path: String,
    file: String,
    staged: bool,
    untracked: bool,
) -> Result<GitDiff, String> {
    git_service::file_diff(&path, &file, staged, untracked).map_err(log_err)
}

#[tauri::command]
pub fn git_commit_file_diff(
    path: String,
    hash: String,
    file: String,
) -> Result<GitDiff, String> {
    git_service::commit_file_diff(&path, &hash, &file).map_err(log_err)
}

#[tauri::command]
pub fn git_log(path: String, limit: u32) -> Result<Vec<GitCommit>, String> {
    git_service::log(&path, limit).map_err(log_err)
}

#[tauri::command]
pub fn git_commit_detail(path: String, hash: String) -> Result<GitCommitDetail, String> {
    git_service::commit_detail(&path, &hash).map_err(log_err)
}

#[tauri::command]
pub fn git_branches(path: String) -> Result<Vec<GitBranch>, String> {
    git_service::branches(&path).map_err(log_err)
}

#[tauri::command]
pub fn git_stash_list(path: String) -> Result<Vec<GitStash>, String> {
    git_service::stash_list(&path).map_err(log_err)
}

#[tauri::command]
pub fn git_worktree_list(path: String) -> Result<Vec<GitWorktree>, String> {
    git_service::worktree_list(&path).map_err(log_err)
}

// === Thao tác ghi / mạng (async) ===

#[tauri::command]
pub async fn git_stage(path: String, files: Vec<String>) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || git_service::stage(&path, &files))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_unstage(path: String, files: Vec<String>) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || git_service::unstage(&path, &files))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_discard(path: String, files: Vec<String>) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || git_service::discard(&path, &files))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_commit(path: String, message: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::commit(&path, &message))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_checkout_branch(path: String, name: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::checkout_branch(&path, &name))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_create_branch(
    path: String,
    name: String,
    from: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::create_branch(&path, &name, &from))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_delete_branch(
    path: String,
    name: String,
    force: bool,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::delete_branch(&path, &name, force))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_fetch(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::fetch(&path))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_pull(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::pull(&path))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_push(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::push(&path))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_stash_save(path: String, message: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::stash_save(&path, &message))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_stash_apply(
    path: String,
    reference: String,
    pop: bool,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::stash_apply(&path, &reference, pop))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_stash_drop(path: String, reference: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::stash_drop(&path, &reference))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_clone(url: String, dest: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::clone(&url, &dest))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_undo_last_commit(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::undo_last_commit(&path))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_reset(path: String, hash: String, mode: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::reset_to(&path, &hash, &mode))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_revert(path: String, hash: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::revert(&path, &hash))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_revert_abort(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::revert_abort(&path))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_rebase(path: String, onto: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::rebase(&path, &onto))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_rebase_abort(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::rebase_abort(&path))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_rebase_continue(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::rebase_continue(&path))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_cherry_pick(path: String, hash: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::cherry_pick(&path, &hash))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_cherry_pick_abort(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::cherry_pick_abort(&path))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_cherry_pick_continue(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || git_service::cherry_pick_continue(&path))
        .await
        .map_err(log_err)?
        .map_err(log_err)
}

#[tauri::command]
pub async fn git_worktree_add(
    path: String,
    worktree_path: String,
    branch: String,
    new_branch: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        git_service::worktree_add(&path, &worktree_path, &branch, &new_branch)
    })
    .await
    .map_err(log_err)?
    .map_err(log_err)
}

#[tauri::command]
pub async fn git_worktree_remove(
    path: String,
    worktree_path: String,
    force: bool,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        git_service::worktree_remove(&path, &worktree_path, force)
    })
    .await
    .map_err(log_err)?
    .map_err(log_err)
}
