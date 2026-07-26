import { Channel } from "@tauri-apps/api/core";
import { safeInvoke } from "./_base";

/**
 * Mở một phiên terminal (PTY) mới ở backend và trả về id phiên.
 *
 * Output và tín hiệu kết thúc được đẩy về qua `ipc::Channel` (đường IPC trực
 * tiếp, độ trễ thấp) thay vì event bus `emit`/`listen` — tránh lag gõ phím trên
 * macOS/WKWebView. Mỗi phiên có kênh riêng nên không cần định tuyến theo id ở
 * phía frontend: callback gắn thẳng vào terminal tương ứng.
 *
 * @param rows/cols kích thước khởi tạo (số hàng/cột) của terminal.
 * @param onOutput nhận từng chunk output dạng base64 (giải mã rồi nạp vào xterm).
 * @param onExit gọi khi tiến trình shell kết thúc, kèm mã thoát (hoặc null).
 * @param cwd thư mục làm việc ban đầu (bỏ trống → home directory).
 * @param shell đường dẫn shell (bỏ trống → shell mặc định của OS).
 */
export function terminalSpawn(
  rows: number,
  cols: number,
  onOutput: (dataBase64: string) => void,
  onExit: (code: number | null) => void,
  cwd?: string,
  shell?: string,
) {
  const outputChannel = new Channel<string>();
  outputChannel.onmessage = onOutput;
  const exitChannel = new Channel<number | null>();
  exitChannel.onmessage = onExit;
  return safeInvoke<string>("terminal_spawn", {
    rows,
    cols,
    cwd,
    shell,
    outputChannel,
    exitChannel,
  });
}

/** Gửi dữ liệu người dùng gõ vào shell của phiên `id`. */
export function terminalWrite(id: string, data: string) {
  return safeInvoke<void>("terminal_write", { id, data });
}

/** Đổi kích thước (số hàng/cột) của phiên `id`. */
export function terminalResize(id: string, rows: number, cols: number) {
  return safeInvoke<void>("terminal_resize", { id, rows, cols });
}

/** Kết thúc phiên `id` (idempotent — không lỗi nếu phiên đã đóng). */
export function terminalKill(id: string) {
  return safeInvoke<void>("terminal_kill", { id });
}
