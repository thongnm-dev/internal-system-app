import type { AiTaskResult } from "@/tauri/commands/ai-task";

export type AiCoworkState = {
  /** Thư mục project làm việc gần nhất. */
  project_dir: string;
  /** Danh sách task đang hiển thị ở cột "Tasks" (giữ nguyên object để khôi phục y hệt). */
  selected_tasks: AiTaskResult[];
  /** Id các task đã tick chọn (checkbox xác nhận). */
  confirmed_task_ids: number[];
  /** Workflow đang chọn ở dropdown (chưa chắc đã Apply). */
  selected_workflow_id: number | null;
  /** Workflow đã Apply (đang áp dụng, có nạp danh sách step). */
  applied_workflow_id: number | null;
};
