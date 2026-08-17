import { open, save } from "@tauri-apps/plugin-dialog";
import { computed, ref } from "vue";
import type {
  ApplyResult,
  CleanupResult,
  ConfirmedInsert,
  RedCellVerification,
  RowAlignmentReport,
  RowAlignmentSuggestion,
  SyncAnalysis,
} from "@/_/types/vnjp-sync";
import { useToast } from "@/shared/composables/useToast";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import {
  vnjpSyncAnalyze,
  vnjpSyncAnalyzeRowAlignment,
  vnjpSyncApply,
  vnjpSyncCleanup,
  vnjpSyncExportReport,
  vnjpSyncInsertRows,
  vnjpSyncVerifyRedCellsAi,
} from "@/tauri/commands/vnjp-sync";

/** Nhà cung cấp/model AI mặc định dùng cho bước kiểm tra ô đỏ tự động (ưu tiên nhanh, rẻ). */
const AI_VERIFY_PROVIDER = "gemini";
const AI_VERIFY_MODEL = "gemini-3.1-flash-lite";

export type ActiveTab = "overview" | "red-cells" | "strike-cells" | "quality";

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
  const applying = ref(false);
  const exporting = ref(false);
  const cleaning = ref(false);
  const error = ref("");
  const activeTab = ref<ActiveTab>("overview");
  const applyResult = ref<ApplyResult | null>(null);
  const cleanupResult = ref<CleanupResult | null>(null);
  const rowAlignment = ref<RowAlignmentReport | null>(null);
  const checkingAlignment = ref(false);
  const insertingRows = ref(false);
  const confirmedInserts = ref<Set<string>>(new Set());
  const verifyingAi = ref(false);
  const redCellVerifications = ref<Map<string, RedCellVerification>>(new Map());

  const vnName = computed(() => (vnPath.value ? basename(vnPath.value) : ""));
  const jpName = computed(() => (jpPath.value ? basename(jpPath.value) : ""));
  const canAnalyze = computed(
    () => vnPath.value !== "" && jpPath.value !== "" && !analyzing.value,
  );
  const canApply = computed(
    () =>
      analysis.value !== null &&
      analysis.value.redCells.length > 0 &&
      !applying.value,
  );
  const canExport = computed(() => analysis.value !== null && !exporting.value);
  const canCleanup = computed(() => jpPath.value !== "" && !cleaning.value);
  const canCheckAlignment = computed(
    () => vnPath.value !== "" && jpPath.value !== "" && !checkingAlignment.value,
  );
  const hasConfirmedInserts = computed(() => confirmedInserts.value.size > 0);

  const totalRedCells = computed(() => analysis.value?.redCells.length ?? 0);
  const totalStrikeCells = computed(
    () => analysis.value?.strikeCells.length ?? 0,
  );
  const totalQualityIssues = computed(
    () => analysis.value?.qualityIssues.length ?? 0,
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
      cleanupResult.value = null;
      rowAlignment.value = null;
      confirmedInserts.value = new Set();
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
    cleanupResult.value = null;
    rowAlignment.value = null;
    confirmedInserts.value = new Set();
  }

  function clearFile(slot: "vn" | "jp") {
    if (slot === "vn") {
      vnPath.value = "";
    } else {
      jpPath.value = "";
    }
    analysis.value = null;
    applyResult.value = null;
    cleanupResult.value = null;
    rowAlignment.value = null;
    confirmedInserts.value = new Set();
    redCellVerifications.value = new Map();
    error.value = "";
  }

  async function analyze() {
    if (!canAnalyze.value) return;
    error.value = "";
    analyzing.value = true;
    applyResult.value = null;
    redCellVerifications.value = new Map();
    try {
      analysis.value = await vnjpSyncAnalyze(vnPath.value, jpPath.value);
      activeTab.value = "overview";
      toast.success(
        `Phân tích xong: ${analysis.value.redCells.length} ô đỏ, ${analysis.value.strikeCells.length} ô strikethrough`,
      );
      if (analysis.value.redCells.length > 0) {
        // Chạy nền, không chặn nút Phân tích — kết quả điền dần vào badge ở tab Ô đỏ.
        void verifyRedCellsAi();
      }
    } catch (e) {
      error.value = friendlyError(e);
      analysis.value = null;
      toast.error(error.value);
    } finally {
      analyzing.value = false;
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

  async function applyChanges() {
    if (!canApply.value) return;
    if (!canUseTauriRuntime()) return;
    error.value = "";
    try {
      const stem =
        basename(jpPath.value).replace(/\.[^.]+$/, "") || "jp-updated";
      const ext =
        jpPath.value.split(".").pop()?.toLowerCase() === "xlsm"
          ? "xlsm"
          : "xlsx";
      const outPath = await save({
        defaultPath: `${stem}_updated.${ext}`,
        filters: [{ name: "Excel", extensions: ["xlsx", "xlsm"] }],
      });
      if (!outPath) return;
      applying.value = true;

      const result = await vnjpSyncApply(vnPath.value, jpPath.value, outPath);
      applyResult.value = result;
      const cleanupNote =
        result.strikeRemovedCount > 0 || result.redBlackenedCount > 0
          ? ` Đã dọn dẹp: xóa ${result.strikeRemovedCount} ô strikethrough cũ, tô đen ${result.redBlackenedCount} ô chữ đỏ cũ.`
          : "";
      toast.success(
        `Đã ghi ${result.appliedCount} ô VN vào file JP (${result.sheetsModified.length} sheet).${cleanupNote} Mở file để dịch từng ô đỏ.`,
      );
    } catch (e) {
      error.value = friendlyError(e);
      toast.error(error.value);
    } finally {
      applying.value = false;
    }
  }

  /** Dọn dẹp file JP (xóa strikethrough cũ, tô đen chữ đỏ cũ) mà không phản ánh chữ đỏ VN — dùng để xem trước/kiểm tra riêng bước dọn dẹp. */
  async function cleanupJp() {
    if (!canCleanup.value) return;
    if (!canUseTauriRuntime()) return;
    error.value = "";
    try {
      const stem = basename(jpPath.value).replace(/\.[^.]+$/, "") || "jp-cleaned";
      const ext = jpPath.value.split(".").pop()?.toLowerCase() === "xlsm" ? "xlsm" : "xlsx";
      const outPath = await save({
        defaultPath: `${stem}_cleaned.${ext}`,
        filters: [{ name: "Excel", extensions: ["xlsx", "xlsm"] }],
      });
      if (!outPath) return;
      cleaning.value = true;

      const result = await vnjpSyncCleanup(jpPath.value, outPath);
      cleanupResult.value = result;
      toast.success(
        `Đã dọn dẹp file JP: xóa ${result.strikeRemovedCount} ô strikethrough cũ, tô đen ${result.redBlackenedCount} ô chữ đỏ cũ (${result.sheetsModified.length} sheet).`,
      );
    } catch (e) {
      error.value = friendlyError(e);
      toast.error(error.value);
    } finally {
      cleaning.value = false;
    }
  }

  function insertKey(s: { sheet: string; jpInsertAfterRow: number }): string {
    return `${s.sheet}::${s.jpInsertAfterRow}`;
  }

  function isConfirmed(s: RowAlignmentSuggestion): boolean {
    return confirmedInserts.value.has(insertKey(s));
  }

  function toggleConfirm(s: RowAlignmentSuggestion) {
    const key = insertKey(s);
    const next = new Set(confirmedInserts.value);
    if (next.has(key)) {
      next.delete(key);
    } else {
      next.add(key);
    }
    confirmedInserts.value = next;
  }

  /** Kiểm tra lệch dòng VN↔JP (dòng VN có mà JP chưa có) — dùng ô neo số/mã để canh dòng, không tự chèn. */
  async function checkRowAlignment() {
    if (!canCheckAlignment.value) return;
    if (!canUseTauriRuntime()) return;
    error.value = "";
    checkingAlignment.value = true;
    confirmedInserts.value = new Set();
    try {
      rowAlignment.value = await vnjpSyncAnalyzeRowAlignment(vnPath.value, jpPath.value);
      if (rowAlignment.value.suggestions.length === 0) {
        toast.success("Không phát hiện lệch dòng nào giữa VN và JP.");
      } else {
        toast.success(
          `Phát hiện ${rowAlignment.value.suggestions.length} vị trí lệch dòng — vui lòng xem lại và xác nhận trước khi chèn.`,
        );
      }
    } catch (e) {
      error.value = friendlyError(e);
      rowAlignment.value = null;
      toast.error(error.value);
    } finally {
      checkingAlignment.value = false;
    }
  }

  /** Chèn dòng trống vào file JP tại các vị trí đã được TL xác nhận (checkbox), lưu ra file mới và chuyển sang dùng file đó. */
  async function insertConfirmedRows() {
    if (!rowAlignment.value || confirmedInserts.value.size === 0) return;
    if (!canUseTauriRuntime()) return;
    error.value = "";
    const inserts: ConfirmedInsert[] = rowAlignment.value.suggestions
      .filter((s) => isConfirmed(s))
      .map((s) => ({
        sheet: s.sheet,
        jpInsertAfterRow: s.jpInsertAfterRow,
        insertCount: s.insertCount,
      }));
    if (inserts.length === 0) return;
    try {
      const stem = basename(jpPath.value).replace(/\.[^.]+$/, "") || "jp-aligned";
      const ext = jpPath.value.split(".").pop()?.toLowerCase() === "xlsm" ? "xlsm" : "xlsx";
      const outPath = await save({
        defaultPath: `${stem}_aligned.${ext}`,
        filters: [{ name: "Excel", extensions: ["xlsx", "xlsm"] }],
      });
      if (!outPath) return;
      insertingRows.value = true;

      const result = await vnjpSyncInsertRows(jpPath.value, outPath, inserts);
      toast.success(
        `Đã chèn ${result.rowsInserted} dòng vào ${result.sheetsModified.length} sheet. Đã chuyển sang dùng file JP mới, hãy Phân tích lại.`,
      );
      jpPath.value = outPath;
      rowAlignment.value = null;
      confirmedInserts.value = new Set();
      analysis.value = null;
      applyResult.value = null;
      cleanupResult.value = null;
      redCellVerifications.value = new Map();
    } catch (e) {
      error.value = friendlyError(e);
      toast.error(error.value);
    } finally {
      insertingRows.value = false;
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
    cleanupResult.value = null;
    rowAlignment.value = null;
    confirmedInserts.value = new Set();
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
    applying,
    exporting,
    cleaning,
    checkingAlignment,
    verifyingAi,
    insertingRows,
    error,
    activeTab,
    applyResult,
    cleanupResult,
    rowAlignment,
    hasConfirmedInserts,
    totalRedCells,
    totalStrikeCells,
    totalQualityIssues,
    canAnalyze,
    canApply,
    canExport,
    canCleanup,
    canCheckAlignment,
    pickFile,
    dropFile,
    clearFile,
    analyze,
    applyChanges,
    cleanupJp,
    checkRowAlignment,
    insertConfirmedRows,
    isConfirmed,
    toggleConfirm,
    getVerification,
    exportReport,
    reset,
  };
}
