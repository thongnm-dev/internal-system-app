/** Loại tài khoản AI — backend tự detect từ prefix của API key. */
export type AiAccountType = "api" | "admin" | "oauth" | "unknown";

/** Một account AI đã đăng ký (backend trả về, API key đã che). */
export type AiAccount = {
  id: number;
  name: string;
  api_key_masked: string;
  account_type: AiAccountType;
  /** Phần trăm usage còn lại (0–100). */
  usage_percent: number;
  /** Thời điểm reset usage tiếp theo (`YYYY-MM-DD HH:MM:SS`). */
  reset_at: string;
  /** Số session đã mở với account này. */
  session_count: number;
  created_at: string;
};

/** Request thêm account AI. */
export type AddAiAccountRequest = {
  name: string;
  api_key: string;
};
