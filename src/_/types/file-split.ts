/** Kiểu dữ liệu cho công cụ nén ZIP (AES-256) và cắt file `.001`. */

/** Một nguồn (file/folder) gửi xuống backend để nén. */
export interface FileSplitSourceInput {
  path: string;
  name: string;
}

/** Thông số nén + tách gửi xuống backend (khớp `FileSplitOptions` phía Rust). */
export interface FileSplitOptions {
  sources: FileSplitSourceInput[];
  outputDir: string;
  archiveName: string;
  /** Giới hạn mỗi phần (MB). null = không tách. */
  limitMb: number | null;
  /** Mật khẩu zip; "" = không mã hoá. */
  password: string;
  singleArchive: boolean;
}

/** Một file kết quả (zip nguyên hoặc một phần `.001`). */
export interface FileSplitPart {
  name: string;
  sizeBytes: number;
}

/** Kết quả trả về từ backend sau khi nén + tách. */
export interface FileSplitResult {
  outputDir: string;
  files: FileSplitPart[];
  totalBytes: number;
  archiveCount: number;
  splitCount: number;
  encrypted: boolean;
}
