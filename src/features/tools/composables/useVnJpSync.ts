import { open, save } from "@tauri-apps/plugin-dialog";
import { computed, ref } from "vue";
import type { ApplyResult, SyncAnalysis } from "@/_/types/vnjp-sync";
import { useToast } from "@/shared/composables/useToast";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import {
  vnjpSyncAnalyze,
  vnjpSyncApply,
  vnjpSyncExportReport,
} from "@/tauri/commands/vnjp-sync";

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
  const error = ref("");
  const activeTab = ref<ActiveTab>("overview");
  const applyResult = ref<ApplyResult | null>(null);

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
    } catch (e) {
      error.value = friendlyError(e);
    }
  }

  function clearFile(slot: "vn" | "jp") {
    if (slot === "vn") {
      vnPath.value = "";
    } else {
      jpPath.value = "";
    }
    analysis.value = null;
    applyResult.value = null;
    error.value = "";
  }

  async function analyze() {
    if (!canAnalyze.value) return;
    error.value = "";
    analyzing.value = true;
    applyResult.value = null;
    try {
      analysis.value = await vnjpSyncAnalyze(vnPath.value, jpPath.value);
      activeTab.value = "overview";
      toast.success(
        `Phân tích xong: ${analysis.value.redCells.length} ô đỏ, ${analysis.value.strikeCells.length} ô strikethrough`,
      );
    } catch (e) {
      error.value = friendlyError(e);
      analysis.value = null;
      toast.error(error.value);
    } finally {
      analyzing.value = false;
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
      toast.success(
        `Đã ghi ${result.appliedCount} ô VN vào file JP (${result.sheetsModified.length} sheet). Mở file để dịch từng ô đỏ.`,
      );
    } catch (e) {
      error.value = friendlyError(e);
      toast.error(error.value);
    } finally {
      applying.value = false;
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
    error,
    activeTab,
    applyResult,
    totalRedCells,
    totalStrikeCells,
    totalQualityIssues,
    canAnalyze,
    canApply,
    canExport,
    pickFile,
    clearFile,
    analyze,
    applyChanges,
    exportReport,
    reset,
  };
}
