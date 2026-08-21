import { open, save } from "@tauri-apps/plugin-dialog";
import { computed, ref } from "vue";
import type { ApplyResult, RedCellVerification, SyncAnalysis } from "@/_/types/vnjp-sync";
import { useToast } from "@/shared/composables/useToast";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import type { FileEntry } from "@/tauri/commands/explorer";
import { explorerDelete, explorerPaste, explorerReadDir } from "@/tauri/commands/explorer";
import {
  vnjpSyncAnalyzeAndApply,
  vnjpSyncExportReport,
  vnjpSyncTempDir,
  vnjpSyncVerifyRedCellsAi,
} from "@/tauri/commands/vnjp-sync";

/** Nhà cung cấp/model AI mặc định dùng cho bước kiểm tra ô đỏ tự động (ưu tiên nhanh, rẻ). */
const AI_VERIFY_PROVIDER = "gemini";
const AI_VERIFY_MODEL = "gemini-3.1-flash-lite";

export type ActiveTab = "overview" | "red-cells" | "quality" | "data-mismatches";

const XLSX_FILTER = [{ name: "Excel", extensions: ["xlsx", "xlsm"] }];

function basename(path: string) {
  return path.split(/[/\\]/).pop() ?? path;
}

export function useVnJpSync() {
  const toast = useToast();

  const vnPath = ref("");
  const jpPath = ref("");
  const analysis = ref<SyncAnalysis | null>(null);
  const analyzing = ref(false);
  const exporting = ref(false);
  const error = ref("");
  const activeTab = ref<ActiveTab>("overview");
  const applyResult = ref<ApplyResult | null>(null);
  const verifyingAi = ref(false);
  const redCellVerifications = ref<Map<string, RedCellVerification>>(new Map());
  // Thư mục Temp chứa mọi file kết quả đã tạo ra (không chỉ lượt Áp dụng gần nhất) — để TL tự mở
  // hoặc copy sang thư mục làm việc khác. Đường dẫn cache lại sau lần đọc đầu tiên.
  const tempDirPath = ref("");
  const tempFiles = ref<FileEntry[]>([]);
  const loadingTempFiles = ref(false);
  const clearingTempFiles = ref(false);
  const copyingTempFiles = ref(false);
  const selectedTempFilePaths = ref<Set<string>>(new Set());

  const vnName = computed(() => (vnPath.value ? basename(vnPath.value) : ""));
  const jpName = computed(() => (jpPath.value ? basename(jpPath.value) : ""));
  // Dùng cho nút gộp "Phân tích & Áp dụng" — backend chạy cả 2 bước trong 1 lệnh gọi duy nhất
  // (xem `vnjp_sync_analyze_and_apply`).
  const canAnalyzeAndApply = computed(
    () => vnPath.value !== "" && jpPath.value !== "" && !analyzing.value,
  );
  const canExport = computed(() => analysis.value !== null && !exporting.value);
  const selectedTempFiles = computed(() =>
    tempFiles.value.filter((f) => selectedTempFilePaths.value.has(f.path)),
  );
  const hasSelectedTempFiles = computed(() => selectedTempFilePaths.value.size > 0);

  const totalRedCells = computed(() => analysis.value?.redCells.length ?? 0);
  const totalQualityIssues = computed(
    () => analysis.value?.qualityIssues.length ?? 0,
  );
  const totalDataMismatches = computed(
    () => applyResult.value?.dataMismatches.length ?? 0,
  );

  async function pickFile(slot: "vn" | "jp") {
    if (!canUseTauriRuntime()) return;
    error.value = "";
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: XLSX_FILTER,
      });
      if (typeof selected !== "string") return;
      if (slot === "vn") {
        vnPath.value = selected;
      } else {
        jpPath.value = selected;
      }
      analysis.value = null;
      applyResult.value = null;
      redCellVerifications.value = new Map();
    } catch (e) {
      error.value = friendlyError(e);
    }
  }

  /** Kéo-thả file từ ngoài vào ô VN/JP — path lấy trực tiếp từ Tauri drag-drop, không qua dialog. */
  function dropFile(slot: "vn" | "jp", path: string) {
    error.value = "";
    const ext = path.split(".").pop()?.toLowerCase() ?? "";
    if (ext !== "xlsx" && ext !== "xlsm") {
      error.value = "Định dạng không hỗ trợ. Chỉ nhận file Excel (.xlsx / .xlsm).";
      return;
    }
    if (slot === "vn") {
      vnPath.value = path;
    } else {
      jpPath.value = path;
    }
    analysis.value = null;
    applyResult.value = null;
  }

  function clearFile(slot: "vn" | "jp") {
    if (slot === "vn") {
      vnPath.value = "";
    } else {
      jpPath.value = "";
    }
    analysis.value = null;
    applyResult.value = null;
    redCellVerifications.value = new Map();
    error.value = "";
  }

  /**
   * Phân tích khác biệt VN/JP RỒI áp dụng luôn trong cùng 1 lệnh gọi backend duy nhất (xem
   * `vnjp_sync_analyze_and_apply`) — không còn 2 bước/2 command riêng ở frontend.
   */
  async function analyzeAndApply() {
    if (!canAnalyzeAndApply.value) return;
    if (!canUseTauriRuntime()) return;
    error.value = "";
    analyzing.value = true;
    applyResult.value = null;
    redCellVerifications.value = new Map();
    try {
      const result = await vnjpSyncAnalyzeAndApply(vnPath.value, jpPath.value);
      analysis.value = result.analysis;
      applyResult.value = result.apply;
      activeTab.value = "overview";

      if (result.analysis.redCells.length > 0) {
        // Chạy nền, không chặn nút — kết quả điền dần vào badge ở tab Ô đỏ.
        void verifyRedCellsAi();
      }
      void refreshTempFiles();
    } catch (e) {
      error.value = friendlyError(e);
      analysis.value = null;
      applyResult.value = null;
      toast.error(error.value);
    } finally {
      analyzing.value = false;
    }
  }

  /** Liệt kê file trong thư mục Temp (nơi lưu kết quả "Áp dụng") — cho TL tự mở/copy sang nơi khác. */
  async function refreshTempFiles() {
    if (!canUseTauriRuntime()) return;
    loadingTempFiles.value = true;
    try {
      if (!tempDirPath.value) {
        tempDirPath.value = await vnjpSyncTempDir();
      }
      const result = await explorerReadDir(tempDirPath.value);
      tempFiles.value = result.entries
        .filter((e) => !e.is_dir)
        .sort((a, b) => b.modified.localeCompare(a.modified));
      // Bỏ chọn các file không còn tồn tại sau khi làm mới (đã bị xóa/di chuyển).
      const stillThere = new Set(tempFiles.value.map((f) => f.path));
      selectedTempFilePaths.value = new Set(
        [...selectedTempFilePaths.value].filter((p) => stillThere.has(p)),
      );
    } catch {
      // Im lặng bỏ qua — chỉ là danh sách hỗ trợ, không phải hành động TL chủ động bấm.
    } finally {
      loadingTempFiles.value = false;
    }
  }

  function isTempFileSelected(path: string): boolean {
    return selectedTempFilePaths.value.has(path);
  }

  function toggleTempFileSelection(path: string) {
    const next = new Set(selectedTempFilePaths.value);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    selectedTempFilePaths.value = next;
  }

  /** Xóa toàn bộ file đang liệt kê trong thư mục Temp — dùng khi TL muốn dọn sạch trước khi bắt đầu lượt mới. */
  async function clearAllTempFiles() {
    if (tempFiles.value.length === 0) return;
    if (!canUseTauriRuntime()) return;
    error.value = "";
    clearingTempFiles.value = true;
    try {
      const paths = tempFiles.value.map((f) => f.path);
      await explorerDelete(paths);
      toast.success(`Đã xóa ${paths.length} file trong thư mục Temp.`);
      await refreshTempFiles();
    } catch (e) {
      error.value = friendlyError(e);
      toast.error(error.value);
    } finally {
      clearingTempFiles.value = false;
    }
  }

  /** Xóa các file đang được chọn (checkbox) trong thư mục Temp. */
  async function deleteSelectedTempFiles() {
    if (selectedTempFilePaths.value.size === 0) return;
    if (!canUseTauriRuntime()) return;
    error.value = "";
    clearingTempFiles.value = true;
    try {
      const paths = [...selectedTempFilePaths.value];
      await explorerDelete(paths);
      toast.success(`Đã xóa ${paths.length} file đã chọn.`);
      await refreshTempFiles();
    } catch (e) {
      error.value = friendlyError(e);
      toast.error(error.value);
    } finally {
      clearingTempFiles.value = false;
    }
  }

  /** Copy các file đang được chọn (checkbox) sang thư mục đích do TL tự chọn. */
  async function copySelectedTempFiles(destDir: string) {
    if (!destDir || selectedTempFilePaths.value.size === 0) return;
    if (!canUseTauriRuntime()) return;
    error.value = "";
    copyingTempFiles.value = true;
    try {
      const paths = [...selectedTempFilePaths.value];
      await explorerPaste(paths, destDir, false);
      toast.success(`Đã copy ${paths.length} file sang ${destDir}.`);
      selectedTempFilePaths.value = new Set();
    } catch (e) {
      error.value = friendlyError(e);
      toast.error(error.value);
      throw e;
    } finally {
      copyingTempFiles.value = false;
    }
  }

  function verificationKey(sheet: string, row: number, col: number): string {
    return `${sheet}::${row}::${col}`;
  }

  /** Lấy kết quả kiểm tra AI (nếu đã có) cho 1 ô đỏ cụ thể — dùng để hiển thị badge. */
  function getVerification(sheet: string, row: number, col: number): RedCellVerification | undefined {
    return redCellVerifications.value.get(verificationKey(sheet, row, col));
  }

  /**
   * Dịch VN→JP (chỉ để so sánh, KHÔNG ghi vào tài liệu) cho toàn bộ ô đỏ vừa phân tích, rồi so
   * độ tương đồng với nội dung JP hiện có — tự động chạy nền sau mỗi lần Phân tích. Nếu AI chưa
   * cấu hình (thiếu API key) hoặc lỗi mạng thì bỏ qua lặng lẽ, không làm phiền bằng toast lỗi.
   */
  async function verifyRedCellsAi() {
    if (!analysis.value || analysis.value.redCells.length === 0) return;
    if (!canUseTauriRuntime()) return;
    verifyingAi.value = true;
    try {
      const report = await vnjpSyncVerifyRedCellsAi(
        jpPath.value,
        analysis.value.redCells,
        AI_VERIFY_PROVIDER,
        AI_VERIFY_MODEL,
      );
      const next = new Map<string, RedCellVerification>();
      for (const item of report.items) {
        next.set(verificationKey(item.sheet, item.row, item.col), item);
      }
      redCellVerifications.value = next;
    } catch {
      // Im lặng bỏ qua — đây là bước hỗ trợ chạy nền, không phải hành động TL chủ động bấm.
    } finally {
      verifyingAi.value = false;
    }
  }

  async function exportReport() {
    if (!canExport.value || !analysis.value) return;
    if (!canUseTauriRuntime()) return;
    error.value = "";
    try {
      const stem =
        basename(jpPath.value).replace(/\.[^.]+$/, "") || "vnjp-sync";
      const path = await save({
        defaultPath: `${stem}_sync-report.xlsx`,
        filters: [{ name: "Excel", extensions: ["xlsx"] }],
      });
      if (!path) return;
      exporting.value = true;
      await vnjpSyncExportReport(analysis.value, path);
      toast.success("Đã xuất báo cáo phân tích thành công.");
    } catch (e) {
      error.value = friendlyError(e);
      toast.error(error.value);
    } finally {
      exporting.value = false;
    }
  }

  function reset() {
    vnPath.value = "";
    jpPath.value = "";
    analysis.value = null;
    applyResult.value = null;
    redCellVerifications.value = new Map();
    error.value = "";
    activeTab.value = "overview";
  }

  return {
    vnPath,
    jpPath,
    vnName,
    jpName,
    analysis,
    analyzing,
    exporting,
    verifyingAi,
    error,
    activeTab,
    applyResult,
    tempDirPath,
    tempFiles,
    loadingTempFiles,
    clearingTempFiles,
    copyingTempFiles,
    selectedTempFiles,
    hasSelectedTempFiles,
    totalRedCells,
    totalQualityIssues,
    totalDataMismatches,
    canAnalyzeAndApply,
    canExport,
    pickFile,
    dropFile,
    clearFile,
    analyzeAndApply,
    refreshTempFiles,
    clearAllTempFiles,
    deleteSelectedTempFiles,
    isTempFileSelected,
    toggleTempFileSelection,
    copySelectedTempFiles,
    getVerification,
    exportReport,
    reset,
  };
}
