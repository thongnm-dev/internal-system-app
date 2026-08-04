import { open, save } from "@tauri-apps/plugin-dialog";
import { computed, ref } from "vue";
import type { CompareResult, FileCompareKind } from "@/_/types/file-compare";
import { useToast } from "@/shared/composables/useToast";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { fileCompareExport, fileCompareRun } from "@/tauri/commands/file-compare";

export interface PickedFile {
  path: string;
  name: string;
  kind: FileCompareKind;
}

/** Ánh xạ phần mở rộng → loại file. null nếu không hỗ trợ. */
function kindFromPath(path: string): FileCompareKind | null {
  const ext = path.split(".").pop()?.toLowerCase() ?? "";
  if (ext === "md" || ext === "markdown") return "markdown";
  if (ext === "txt" || ext === "text" || ext === "log" || ext === "csv") return "text";
  if (ext === "docx") return "word";
  if (ext === "xlsx" || ext === "xls" || ext === "xlsm") return "excel";
  return null;
}

const DIALOG_FILTERS = [
  {
    name: "Hỗ trợ so sánh",
    extensions: ["md", "markdown", "txt", "log", "csv", "docx", "xlsx", "xls", "xlsm"],
  },
];

export function useFileCompare() {
  const toast = useToast();
  const fileA = ref<PickedFile | null>(null);
  const fileB = ref<PickedFile | null>(null);
  const result = ref<CompareResult | null>(null);
  const loading = ref(false);
  const exporting = ref(false);
  const error = ref("");

  const commonKind = computed<FileCompareKind | null>(() => {
    if (!fileA.value || !fileB.value) return null;
    return fileA.value.kind === fileB.value.kind ? fileA.value.kind : null;
  });
  const kindMismatch = computed(
    () => !!fileA.value && !!fileB.value && fileA.value.kind !== fileB.value.kind,
  );
  const canCompare = computed(() => !!commonKind.value && !loading.value);

  function applyPickedPath(slot: "a" | "b", path: string): boolean {
    const kind = kindFromPath(path);
    if (!kind) {
      error.value = "Định dạng không hỗ trợ. Chỉ nhận text, markdown, .docx, Excel.";
      return false;
    }
    const name = path.split(/[/\\]/).pop() ?? path;
    const picked: PickedFile = { path, name, kind };
    if (slot === "a") fileA.value = picked;
    else fileB.value = picked;
    result.value = null;
    return true;
  }

  async function pickFile(slot: "a" | "b") {
    error.value = "";
    if (!canUseTauriRuntime()) return;
    try {
      const selected = await open({ multiple: false, directory: false, filters: DIALOG_FILTERS });
      if (typeof selected !== "string") return;
      applyPickedPath(slot, selected);
    } catch (e) {
      error.value = friendlyError(e);
    }
  }

  /** Kéo-thả file từ ngoài vào ô A/B — path lấy trực tiếp từ Tauri drag-drop, không qua dialog. */
  function dropFile(slot: "a" | "b", path: string) {
    error.value = "";
    applyPickedPath(slot, path);
  }

  function clearFile(slot: "a" | "b") {
    if (slot === "a") fileA.value = null;
    else fileB.value = null;
    result.value = null;
  }

  async function compare() {
    if (!fileA.value || !fileB.value) return;
    if (kindMismatch.value) {
      error.value = "2 file phải cùng loại mới so sánh được.";
      return;
    }
    error.value = "";
    loading.value = true;
    try {
      result.value = await fileCompareRun(fileA.value.path, fileB.value.path);
    } catch (e) {
      error.value = friendlyError(e);
      result.value = null;
    } finally {
      loading.value = false;
    }
  }

  /** Xuất kết quả hiện tại ra file Excel (.xlsx) — mở hộp thoại chọn nơi lưu. */
  async function exportExcel() {
    if (!fileA.value || !fileB.value || !result.value) return;
    if (!canUseTauriRuntime()) return;
    error.value = "";
    const stem = fileA.value.name.replace(/\.[^.]+$/, "");
    try {
      const path = await save({
        defaultPath: `so-sanh_${stem}.xlsx`,
        filters: [{ name: "Excel", extensions: ["xlsx"] }],
      });
      if (!path) return;
      exporting.value = true;
      await fileCompareExport(fileA.value.path, fileB.value.path, path);
      toast.success("Đã xuất kết quả so sánh ra Excel.");
    } catch (e) {
      error.value = friendlyError(e);
      toast.error(error.value);
    } finally {
      exporting.value = false;
    }
  }

  function reset() {
    fileA.value = null;
    fileB.value = null;
    result.value = null;
    error.value = "";
  }

  return {
    fileA,
    fileB,
    result,
    loading,
    exporting,
    error,
    commonKind,
    kindMismatch,
    canCompare,
    pickFile,
    dropFile,
    clearFile,
    compare,
    exportExcel,
    reset,
  };
}
