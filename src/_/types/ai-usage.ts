/** Loại tài khoản AI — backend tự detect từ prefix của API key. */
export type AiAccountType = "api" | "admin" | "oauth" | "unknown";

/** Nhà cung cấp AI. Auto-switch chỉ diễn ra trong cùng provider. */
export type AiProvider = "claude" | "codex";

/** Trạng thái usage của account. */
export type AiAccountStatus = "healthy" | "low" | "exhausted" | "error" | "unknown";

/** Nguồn số liệu usage — để UI nói rõ con số đến từ đâu. */
export type AiUsageSource =
  | "billing_api"
  | "ratelimit_header"
  | "error_signal"
  | "manual"
  | "unknown";

/** Một account AI đã đăng ký (backend trả về, API key đã che). */
export type AiAccount = {
  id: number;
  name: string;
  api_key_masked: string;
  account_type: AiAccountType;
  /** Nhà cung cấp: `claude` | `codex`. */
  provider: AiProvider;
  /** Thứ tự ưu tiên — số nhỏ = ưu tiên cao hơn. */
  priority: number;
  /** `true` nếu account đang được chọn (active) cho provider của nó. */
  is_active: boolean;
  status: AiAccountStatus;
  usage_source: AiUsageSource;
  /** Phần trăm usage còn lại (0–100). */
  usage_percent: number;
  /** Thời điểm reset usage tiếp theo (`YYYY-MM-DD HH:MM:SS`, có thể rỗng). */
  reset_at: string;
  /** Số session đã mở với account này. */
  session_count: number;
  /** Lần probe usage gần nhất (`YYYY-MM-DD HH:MM:SS`, có thể rỗng). */
  last_checked_at: string;
  created_at: string;
};

/** Request thêm account AI. */
export type AddAiAccountRequest = {
  name: string;
  api_key: string;
  provider?: AiProvider;
  priority?: number;
};

/** Request cập nhật account (không đổi API key). */
export type UpdateAiAccountRequest = {
  id: number;
  name?: string;
  provider?: AiProvider;
  priority?: number;
};

/** Tín hiệu usage do skill/automation báo về. */
export type ReportUsageSignalRequest = {
  id: number;
  exhausted: boolean;
  reset_at?: string;
};

/** Cấu hình auto-switch + poll nền. */
export type AiUsageSettings = {
  switch_threshold_percent: number;
  poll_interval_secs: number;
};
