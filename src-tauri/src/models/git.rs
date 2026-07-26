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
