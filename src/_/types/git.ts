/** Types cho module Git Desktop — khớp với DTO ở `src-tauri/src/models/git.rs`. */

/** Một repository trong danh sách quản lý (lưu cục bộ). */
export interface GitRepo {
  id: number;
  name: string;
  path: string;
  last_opened: string;
}

/** Trạng thái tổng quan của repo. */
export interface GitRepoInfo {
  path: string;
  current_branch: string;
  detached: boolean;
  upstream: string;
  ahead: number;
  behind: number;
  remote_url: string;
  rebase_in_progress: boolean;
  cherry_pick_in_progress: boolean;
  merge_in_progress: boolean;
}

/** Một tag. */
export interface GitTag {
  name: string;
  target: string;
  subject: string;
  date: string;
}

/** Một commit cho đồ thị visualization (kèm parents + refs). */
export interface GitGraphCommit {
  hash: string;
  short_hash: string;
  subject: string;
  author_name: string;
  relative_date: string;
  parents: string[];
  refs: string[];
}

/** Mốc tiến trình của thao tác mạng (fetch/pull/push/clone). */
export interface GitProgress {
  phase: string;
  percent: number;
  raw: string;
}

/** Một Pull Request / Merge Request lấy từ API host. */
export interface GitPullRequest {
  number: number;
  title: string;
  author: string;
  /** "open" | "closed" | "merged" | "draft" */
  state: string;
  draft: boolean;
  head: string;
  base: string;
  url: string;
  created_at: string;
  updated_at: string;
}

/** Kết quả so sánh 2 branch (Compare + preview Pull Request). */
export interface GitComparison {
  base: string;
  head: string;
  ahead: number;
  behind: number;
  commits: GitCommit[];
  files: GitFileChange[];
  web_url: string;
  pr_url: string;
}

/** Một Git worktree. */
export interface GitWorktree {
  path: string;
  head: string;
  branch: string;
  is_bare: boolean;
  is_detached: boolean;
  is_current: boolean;
}

/** Một file trong danh sách thay đổi. */
export interface GitFileChange {
  path: string;
  orig_path: string;
  /** M/A/D/R/C/U/? */
  status: string;
  untracked: boolean;
}

/** Kết quả `git status`. */
export interface GitStatus {
  staged: GitFileChange[];
  unstaged: GitFileChange[];
}

/** Một dòng trong diff. */
export interface GitDiffLine {
  /** "add" | "del" | "context" | "hunk" | "meta" */
  kind: string;
  content: string;
  old_line: number;
  new_line: number;
}

/** Kết quả diff của một file. */
export interface GitDiff {
  path: string;
  lines: GitDiffLine[];
  is_binary: boolean;
  truncated: boolean;
}

/** Một commit trong lịch sử. */
export interface GitCommit {
  hash: string;
  short_hash: string;
  subject: string;
  author_name: string;
  author_email: string;
  date: string;
  relative_date: string;
}

/** Chi tiết một commit. */
export interface GitCommitDetail {
  commit: GitCommit;
  body: string;
  files: GitFileChange[];
}

/** Một branch (local hoặc remote). */
export interface GitBranch {
  name: string;
  is_current: boolean;
  is_remote: boolean;
  upstream: string;
  last_commit_subject: string;
}

/** Một mục stash. */
export interface GitStash {
  index: number;
  reference: string;
  message: string;
}
