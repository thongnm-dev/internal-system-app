import { Channel } from "@tauri-apps/api/core";
import { safeInvoke } from "./_base";
import type {
  GitBranch,
  GitCommit,
  GitCommitDetail,
  GitComparison,
  GitDiff,
  GitProgress,
  GitPullRequest,
  GitRepo,
  GitRepoInfo,
  GitStash,
  GitStatus,
  GitTag,
  GitWorktree,
} from "@/_/types/git";

/** Tạo Channel nhận tiến trình; gắn callback nếu có. */
function progressChannel(onProgress?: (p: GitProgress) => void) {
  const channel = new Channel<GitProgress>();
  if (onProgress) channel.onmessage = onProgress;
  return channel;
}

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

export function gitWorktreeList(path: string) {
  return safeInvoke<GitWorktree[]>("git_worktree_list", { path });
}

export function gitTagList(path: string) {
  return safeInvoke<GitTag[]>("git_tag_list", { path });
}

export function gitCompareFileDiff(path: string, base: string, head: string, file: string) {
  return safeInvoke<GitDiff>("git_compare_file_diff", { path, base, head, file });
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

export function gitFetch(path: string, onProgress?: (p: GitProgress) => void) {
  return safeInvoke<string>("git_fetch", { path, onProgress: progressChannel(onProgress) });
}

export function gitPull(path: string, onProgress?: (p: GitProgress) => void) {
  return safeInvoke<string>("git_pull", { path, onProgress: progressChannel(onProgress) });
}

export function gitPush(path: string, onProgress?: (p: GitProgress) => void) {
  return safeInvoke<string>("git_push", { path, onProgress: progressChannel(onProgress) });
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

export function gitClone(url: string, dest: string, onProgress?: (p: GitProgress) => void) {
  return safeInvoke<string>("git_clone", { url, dest, onProgress: progressChannel(onProgress) });
}

export function gitUndoLastCommit(path: string) {
  return safeInvoke<string>("git_undo_last_commit", { path });
}

export function gitReset(path: string, hash: string, mode: "soft" | "mixed" | "hard") {
  return safeInvoke<string>("git_reset", { path, hash, mode });
}

export function gitRevert(path: string, hash: string) {
  return safeInvoke<string>("git_revert", { path, hash });
}

export function gitRevertAbort(path: string) {
  return safeInvoke<string>("git_revert_abort", { path });
}

export function gitRebase(path: string, onto: string) {
  return safeInvoke<string>("git_rebase", { path, onto });
}

export function gitRebaseAbort(path: string) {
  return safeInvoke<string>("git_rebase_abort", { path });
}

export function gitRebaseContinue(path: string) {
  return safeInvoke<string>("git_rebase_continue", { path });
}

export function gitTagCreate(
  path: string,
  name: string,
  hash: string,
  message: string,
  annotated: boolean,
  push: boolean,
) {
  return safeInvoke<string>("git_tag_create", { path, name, hash, message, annotated, push });
}

export function gitTagDelete(path: string, name: string, remote: boolean) {
  return safeInvoke<string>("git_tag_delete", { path, name, remote });
}

export function gitMerge(path: string, branch: string, squash: boolean, message: string) {
  return safeInvoke<string>("git_merge", { path, branch, squash, message });
}

export function gitMergeAbort(path: string) {
  return safeInvoke<string>("git_merge_abort", { path });
}

export function gitCompare(path: string, base: string, head: string) {
  return safeInvoke<GitComparison>("git_compare", { path, base, head });
}

export function gitListPullRequests(path: string, state: string) {
  return safeInvoke<GitPullRequest[]>("git_list_pull_requests", { path, state });
}

export function gitOpenUrl(url: string) {
  return safeInvoke<void>("git_open_url", { url });
}

export function gitCreatePullRequest(path: string, base: string, head: string) {
  return safeInvoke<string>("git_create_pull_request", { path, base, head });
}

export function gitCherryPick(path: string, hash: string) {
  return safeInvoke<string>("git_cherry_pick", { path, hash });
}

export function gitCherryPickAbort(path: string) {
  return safeInvoke<string>("git_cherry_pick_abort", { path });
}

export function gitCherryPickContinue(path: string) {
  return safeInvoke<string>("git_cherry_pick_continue", { path });
}

export function gitWorktreeAdd(path: string, worktreePath: string, branch: string, newBranch: string) {
  return safeInvoke<string>("git_worktree_add", { path, worktreePath, branch, newBranch });
}

export function gitWorktreeRemove(path: string, worktreePath: string, force: boolean) {
  return safeInvoke<string>("git_worktree_remove", { path, worktreePath, force });
}
