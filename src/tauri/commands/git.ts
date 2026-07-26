import { safeInvoke } from "./_base";
import type {
  GitBranch,
  GitCommit,
  GitCommitDetail,
  GitDiff,
  GitRepo,
  GitRepoInfo,
  GitStash,
  GitStatus,
} from "@/_/types/git";

// === Quản lý danh sách repo (lưu cục bộ) ===

export function gitListRepos() {
  return safeInvoke<GitRepo[]>("git_list_repos");
}

export function gitAddRepo(path: string) {
  return safeInvoke<GitRepo>("git_add_repo", { path });
}

export function gitRemoveRepo(id: number) {
  return safeInvoke<void>("git_remove_repo", { id });
}

export function gitTouchRepo(id: number) {
  return safeInvoke<void>("git_touch_repo", { id });
}

// === Đọc trạng thái ===

export function gitRepoInfo(path: string) {
  return safeInvoke<GitRepoInfo>("git_repo_info", { path });
}

export function gitStatus(path: string) {
  return safeInvoke<GitStatus>("git_status", { path });
}

export function gitFileDiff(path: string, file: string, staged: boolean, untracked: boolean) {
  return safeInvoke<GitDiff>("git_file_diff", { path, file, staged, untracked });
}

export function gitCommitFileDiff(path: string, hash: string, file: string) {
  return safeInvoke<GitDiff>("git_commit_file_diff", { path, hash, file });
}

export function gitLog(path: string, limit: number) {
  return safeInvoke<GitCommit[]>("git_log", { path, limit });
}

export function gitCommitDetail(path: string, hash: string) {
  return safeInvoke<GitCommitDetail>("git_commit_detail", { path, hash });
}

export function gitBranches(path: string) {
  return safeInvoke<GitBranch[]>("git_branches", { path });
}

export function gitStashList(path: string) {
  return safeInvoke<GitStash[]>("git_stash_list", { path });
}

// === Thao tác ghi / mạng ===

export function gitStage(path: string, files: string[]) {
  return safeInvoke<void>("git_stage", { path, files });
}

export function gitUnstage(path: string, files: string[]) {
  return safeInvoke<void>("git_unstage", { path, files });
}

export function gitDiscard(path: string, files: string[]) {
  return safeInvoke<void>("git_discard", { path, files });
}

export function gitCommit(path: string, message: string) {
  return safeInvoke<string>("git_commit", { path, message });
}

export function gitCheckoutBranch(path: string, name: string) {
  return safeInvoke<string>("git_checkout_branch", { path, name });
}

export function gitCreateBranch(path: string, name: string, from = "") {
  return safeInvoke<string>("git_create_branch", { path, name, from });
}

export function gitDeleteBranch(path: string, name: string, force: boolean) {
  return safeInvoke<string>("git_delete_branch", { path, name, force });
}

export function gitFetch(path: string) {
  return safeInvoke<string>("git_fetch", { path });
}

export function gitPull(path: string) {
  return safeInvoke<string>("git_pull", { path });
}

export function gitPush(path: string) {
  return safeInvoke<string>("git_push", { path });
}

export function gitStashSave(path: string, message: string) {
  return safeInvoke<string>("git_stash_save", { path, message });
}

export function gitStashApply(path: string, reference: string, pop: boolean) {
  return safeInvoke<string>("git_stash_apply", { path, reference, pop });
}

export function gitStashDrop(path: string, reference: string) {
  return safeInvoke<string>("git_stash_drop", { path, reference });
}

export function gitClone(url: string, dest: string) {
  return safeInvoke<string>("git_clone", { url, dest });
}
