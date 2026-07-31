import { safeInvoke } from "./_base";
import type { FileSplitOptions, FileSplitResult } from "@/_/types/file-split";

/** Nén file/folder thành ZIP (AES-256 nếu có mật khẩu) và cắt thành `.001` nếu vượt giới hạn. */
export function fileSplitRun(options: FileSplitOptions) {
  return safeInvoke<FileSplitResult>("file_split_run", { options });
}
