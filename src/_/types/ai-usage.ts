/**
 * Loại tài khoản AI. `api`/`admin`/`oauth` được backend tự detect từ prefix key;
 * `subscription` (Claude Pro/Max qua `claude /login`) do frontend chỉ định.
 */
export type AiAccountType = "api" | "admin" | "oauth" | "subscription" | "unknown";

/** Nhà cung cấp AI. Auto-switch chỉ diễn ra trong cùng provider. */
export type AiProvider = "claude" | "codex";

/**
 * Nguồn account:
 * - `detected` — dò từ login local, token trong Keychain (luôn mới).
 * - `captured` — tool tự capture token → profile trong app data dir (snapshot, có thể hết hạn).
 * - `manual` — nhập API key / tay.
 */
export type AiAccountSource = "detected" | "captured" | "manual";

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
  /** Thư mục `CLAUDE_CONFIG_DIR` account subscription đã login (rỗng với account API key). */
  config_dir: string;
  /** Email account (khi thêm từ detect login local). */
  email: string;
  /** Loại subscription (`team` | `claude_pro` | `max` …). */
  subscription_type: string;
  /** Nguồn account: `detected` | `captured` | `manual`. */
  source: AiAccountSource;
  /** Thứ tự ưu tiên — số nhỏ = ưu tiên cao hơn. */
  priority: number;
  /** `true` nếu account đang được chọn (active) cho provider của nó. */
  is_active: boolean;
  status: AiAccountStatus;
  usage_source: AiUsageSource;
  /** Phần trăm usage còn lại (0–100). Với subscription = min(session, weekly). */
  usage_percent: number;
  /** Thời điểm reset usage tiếp theo (`YYYY-MM-DD HH:MM:SS`, có thể rỗng). */
  reset_at: string;
  /** Session hiện tại (cửa sổ 5 giờ) — phần trăm CÒN LẠI (0–100). */
  session_percent: number;
  /** Thời điểm reset session (`YYYY-MM-DD HH:MM:SS`, rỗng nếu chưa có số liệu). */
  session_reset_at: string;
  /** Weekly limit (cửa sổ 7 ngày) — phần trăm CÒN LẠI (0–100). */
  weekly_percent: number;
  /** Thời điểm reset weekly (`YYYY-MM-DD HH:MM:SS`, rỗng nếu chưa có số liệu). */
  weekly_reset_at: string;
  /** Số session đã mở với account này. */
  session_count: number;
  /** Lần probe usage gần nhất (`YYYY-MM-DD HH:MM:SS`, có thể rỗng). */
  last_checked_at: string;
  created_at: string;
};

/** Request thêm account AI. */
export type AddAiAccountRequest = {
  name: string;
  /** Bỏ trống với account subscription (login qua `claude /login`). */
  api_key?: string;
  provider?: AiProvider;
  /** Chỉ định rõ loại (vd `subscription`); bỏ trống → backend detect từ key. */
  account_type?: AiAccountType;
  /** Thư mục `CLAUDE_CONFIG_DIR` cho account subscription. */
  config_dir?: string;
  /** Email account (khi thêm từ detect login local). */
  email?: string;
  /** Loại subscription (khi thêm từ detect login local). */
  subscription_type?: string;
  /** Nguồn account (`detected` | `captured` | `manual`). Bỏ trống → `manual`. */
  source?: AiAccountSource;
  priority?: number;
};

/** Login Claude đang active trên máy (xem trước để capture — không kèm token). */
export type CapturedLogin = {
  email: string;
  display_name: string;
  subscription_type: string;
  billing_type: string;
  /** Token hết hạn (`YYYY-MM-DD HH:MM:SS`, rỗng nếu không đọc được). */
  token_expires_at: string;
  has_token: boolean;
};

/** Một login Claude phát hiện được trên máy (backend dò từ `.claude.json` + Keychain). */
export type DetectedLogin = {
  /** Thư mục config/data của login (vd `~/.claude` hoặc `<CLAUDE_CONFIG_DIR>`). */
  config_dir: string;
  email: string;
  display_name: string;
  subscription_type: string;
  billing_type: string;
  /** Token hết hạn (`YYYY-MM-DD HH:MM:SS`, rỗng nếu không đọc được Keychain). */
  token_expires_at: string;
  /** Đã có trong danh sách account chưa. */
  already_added: boolean;
};

/** Request cập nhật account (không đổi API key). */
export type UpdateAiAccountRequest = {
  id: number;
  name?: string;
  provider?: AiProvider;
  priority?: number;
  /** Đổi thư mục `CLAUDE_CONFIG_DIR` của account subscription. */
  config_dir?: string;
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
  /** Thư mục project (`CLAUDE_CONFIG_WORK_DIR`) nơi user mở terminal làm việc với AI. */
  work_dir: string;
};
