//! Model/DTO cho module Git Desktop — quản lý repository local và các thao tác Git.
//!
//! Toàn bộ thao tác Git được thực hiện bằng cách gọi `git` CLI của hệ điều hành
//! (giống GitHub Desktop) để tận dụng credential helper sẵn có. Các struct dưới
//! đây là dữ liệu trao đổi giữa backend và frontend qua IPC.

use serde::{Deserialize, Serialize};

/// Một repository đã được thêm vào danh sách quản lý (lưu cục bộ trong JSON).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitRepo {
    pub id: i64,
    /// Tên hiển thị (mặc định là tên thư mục).
    pub name: String,
    /// Đường dẫn tuyệt đối tới thư mục repo.
    pub path: String,
    /// Thời điểm mở gần nhất (ISO string) — dùng để sắp xếp "recent".
    #[serde(default)]
    pub last_opened: String,
}

/// Trạng thái tổng quan của repo (branch hiện tại, ahead/behind, remote).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitRepoInfo {
    /// Đường dẫn thư mục làm việc (top-level).
    pub path: String,
    /// Tên branch hiện tại; rỗng nếu ở trạng thái detached HEAD.
    pub current_branch: String,
    /// Đang ở detached HEAD hay không.
    pub detached: bool,
    /// Tên branch upstream (vd. `origin/main`); rỗng nếu chưa có.
    pub upstream: String,
    /// Số commit local đi trước remote.
    pub ahead: u32,
    /// Số commit local đi sau remote.
    pub behind: u32,
    /// URL remote `origin` (rỗng nếu không có).
    pub remote_url: String,
    /// Đang trong quá trình rebase (có thư mục rebase-merge/rebase-apply).
    pub rebase_in_progress: bool,
    /// Đang trong quá trình cherry-pick (có file CHERRY_PICK_HEAD).
    pub cherry_pick_in_progress: bool,
    /// Đang trong quá trình merge (có file MERGE_HEAD).
    pub merge_in_progress: bool,
}

/// Một tag.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitTag {
    pub name: String,
    /// SHA ngắn của đối tượng tag trỏ tới.
    pub target: String,
    /// Tiêu đề (message annotated tag hoặc subject commit).
    pub subject: String,
    /// Ngày tạo (short).
    pub date: String,
}

/// Một commit dùng cho đồ thị (visualization): kèm parents + refs để dựng lane.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitGraphCommit {
    pub hash: String,
    pub short_hash: String,
    pub subject: String,
    pub author_name: String,
    pub relative_date: String,
    /// Hash các commit cha (parent). Nhiều cha = merge commit.
    pub parents: Vec<String>,
    /// Nhãn tham chiếu (branch/tag), vd. "HEAD -> main", "origin/main", "tag: v1".
    pub refs: Vec<String>,
}

/// Một mốc tiến trình của thao tác mạng (fetch/pull/push/clone), parse từ stderr git.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitProgress {
    /// Giai đoạn (vd. "Receiving objects", "Resolving deltas").
    pub phase: String,
    /// Phần trăm (0-100).
    pub percent: u32,
    /// Dòng gốc để hiển thị chi tiết nếu cần.
    pub raw: String,
}

/// Một Pull Request / Merge Request lấy từ API của host.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitPullRequest {
    pub number: u64,
    pub title: String,
    pub author: String,
    /// "open" | "closed" | "merged" | "draft".
    pub state: String,
    pub draft: bool,
    pub head: String,
    pub base: String,
    pub url: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Kết quả so sánh 2 branch (dùng cho Compare + preview Pull Request).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitComparison {
    pub base: String,
    pub head: String,
    /// Số commit có ở `head` mà chưa có ở `base` (sẽ được đưa vào PR).
    pub ahead: u32,
    /// Số commit có ở `base` mà chưa có ở `head`.
    pub behind: u32,
    /// Danh sách commit `base..head`.
    pub commits: Vec<GitCommit>,
    /// Danh sách file thay đổi `base...head`.
    pub files: Vec<GitFileChange>,
    /// URL web của repo (rỗng nếu không có remote).
    pub web_url: String,
    /// URL tạo Pull Request tương ứng (rỗng nếu không xác định được).
    pub pr_url: String,
}

/// Một Git worktree.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitWorktree {
    /// Đường dẫn thư mục làm việc của worktree.
    pub path: String,
    /// SHA của HEAD.
    pub head: String,
    /// Tên branch đang checkout (rỗng nếu detached/bare).
    pub branch: String,
    /// Worktree bare (không có working tree).
    pub is_bare: bool,
    /// Đang ở detached HEAD.
    pub is_detached: bool,
    /// Là worktree chính đang mở (khớp với repo hiện tại).
    pub is_current: bool,
}

/// Một file trong danh sách thay đổi (working tree hoặc staged).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitFileChange {
    /// Đường dẫn tương đối so với gốc repo.
    pub path: String,
    /// Đường dẫn cũ khi file bị rename (rỗng nếu không rename).
    #[serde(default)]
    pub orig_path: String,
    /// Mã trạng thái 1 ký tự: M/A/D/R/C/U/? (giống git porcelain).
    pub status: String,
    /// File chưa được track (untracked) hay không.
    pub untracked: bool,
}

/// Kết quả `git status`: tách staged và unstaged.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitStatus {
    pub staged: Vec<GitFileChange>,
    pub unstaged: Vec<GitFileChange>,
}

/// Một dòng trong diff (kèm phân loại để tô màu ở frontend).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitDiffLine {
    /// Loại dòng: "add" | "del" | "context" | "hunk" | "meta".
    pub kind: String,
    /// Nội dung dòng (đã bỏ ký tự +/- đầu dòng với add/del để hiển thị gọn).
    pub content: String,
    /// Số dòng phía file cũ (0 nếu không áp dụng).
    pub old_line: u32,
    /// Số dòng phía file mới (0 nếu không áp dụng).
    pub new_line: u32,
}

/// Kết quả diff của một file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitDiff {
    pub path: String,
    pub lines: Vec<GitDiffLine>,
    /// File nhị phân (không hiển thị diff text).
    pub is_binary: bool,
    /// Diff bị cắt bớt do quá lớn.
    pub truncated: bool,
}

/// Một commit trong lịch sử (log).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub short_hash: String,
    pub subject: String,
    pub author_name: String,
    pub author_email: String,
    /// Ngày commit ở dạng ISO (author date).
    pub date: String,
    /// Ngày commit ở dạng tương đối (vd. "2 hours ago").
    pub relative_date: String,
}

/// Chi tiết một commit: thông tin đầy đủ + danh sách file đã đổi.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitCommitDetail {
    pub commit: GitCommit,
    pub body: String,
    pub files: Vec<GitFileChange>,
}

/// Một branch (local hoặc remote).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitBranch {
    pub name: String,
    /// Branch đang được checkout.
    pub is_current: bool,
    /// Branch remote (vd. `origin/main`) hay local.
    pub is_remote: bool,
    /// Upstream tracking branch (rỗng nếu không có).
    #[serde(default)]
    pub upstream: String,
    /// Subject của commit gần nhất trên branch.
    #[serde(default)]
    pub last_commit_subject: String,
}

/// Một dòng trong kết quả `git blame` (ai sửa dòng này lần cuối).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitBlameLine {
    /// Số thứ tự dòng trong file hiện tại (1-based).
    pub line_no: u32,
    pub hash: String,
    pub short_hash: String,
    pub author_name: String,
    /// Ngày commit (ISO string).
    pub date: String,
    /// Ngày commit dạng tương đối (vd. "2 hours ago").
    pub relative_date: String,
    /// Nội dung dòng.
    pub content: String,
}

/// Kết quả `git blame` của một file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitBlame {
    pub path: String,
    pub lines: Vec<GitBlameLine>,
}

/// Một mục trong danh sách stash.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitStash {
    /// Chỉ số stash (0 = mới nhất).
    pub index: u32,
    /// Tham chiếu đầy đủ (vd. `stash@{0}`).
    pub reference: String,
    /// Mô tả stash.
    pub message: String,
}
