import { safeInvoke } from "./_base";

/**
 * Mở một phiên terminal (PTY) mới ở backend và trả về id phiên.
 *
 * @param rows/cols kích thước khởi tạo (số hàng/cột) của terminal.
 * @param cwd thư mục làm việc ban đầu (bỏ trống → home directory).
 * @param shell đường dẫn shell (bỏ trống → shell mặc định của OS).
 */
export function terminalSpawn(
  rows: number,
  cols: number,
  cwd?: string,
  shell?: string,
) {
  return safeInvoke<string>("terminal_spawn", { rows, cols, cwd, shell });
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
