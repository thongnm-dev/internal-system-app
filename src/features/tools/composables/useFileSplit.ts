import { open } from "@tauri-apps/plugin-dialog";
import { computed, reactive, ref } from "vue";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { explorerReadDir } from "@/tauri/commands/explorer";
import { fileSplitCalcSize, fileSplitRun } from "@/tauri/commands/file-split";
import type { MessageMode } from "@/_/types/app";

/** Một mục nguồn được chọn để nén (file hoặc folder). */
export interface FileSplitSource {
  path: string;
  name: string;
  kind: "file" | "folder";
}

/** Thông số cấu hình nén + tách. */
export interface FileSplitConfig {
  outputDir: string;
  archiveName: string;
  limitMb: number | null;
  password: string;
  /** true = không tách, chấp nhận 1 file zip dung lượng lớn. */
  noSplit: boolean;
}

/** Một phần file được tạo ra sau khi tách (kết quả). */
export interface FileSplitPart {
  name: string;
  sizeBytes: number;
}

function baseName(path: string): string {
  const cleaned = path.replace(/[\\/]+$/, "");
  const parts = cleaned.split(/[\\/]/);
  return parts[parts.length - 1] || cleaned;
}

export function useFileSplit() {
  const sources = ref<FileSplitSource[]>([]);

  const config = reactive<FileSplitConfig>({
    outputDir: "",
    archiveName: "attachment",
    limitMb: null,
    password: "",
    noSplit: true,
  });

  const message = ref("Chọn file/folder, thiết lập thông số rồi nhấn Nén & Tách.");
  const messageMode = ref<MessageMode>("info");
  const running = ref(false);
  const parts = ref<FileSplitPart[]>([]);
  const showResult = ref(false);
  const resultMode = ref<"preview" | "run" | null>(null);

  const previewArchives = ref<string[]>([]);
  const previewSizeBytes = ref<number | null>(null);

  const canRun = computed(
    () =>
      sources.value.length > 0 &&
      (config.noSplit || (config.limitMb ?? 0) > 0) &&
      !running.value,
  );

  function normalizedArchiveName(): string {
    const name = config.archiveName.trim().replace(/\.zip$/i, "");
    return name || "attachment";
  }

  async function preview() {
    if (!sources.value.length) {
      setMessage("Chưa có nguồn để xem trước.", "error");
      return;
    }
    previewArchives.value = [`${normalizedArchiveName()}.zip`];
    parts.value = [];
    resultMode.value = "preview";
    showResult.value = true;

    previewSizeBytes.value = null;
    if (canUseTauriRuntime()) {
      try {
        previewSizeBytes.value = await fileSplitCalcSize(sources.value.map((s) => s.path));
      } catch {
        // ignore
      }
    }

    const encNote = config.password ? " · AES-256" : "";
    const splitNote = config.noSplit
      ? " · không tách"
      : ` · tách theo ${config.limitMb ?? 0} MB (số phần .001… tính khi chạy)`;
    setMessage(`Dự kiến 1 file zip${encNote}${splitNote}.`);
  }

  function setMessage(text: string, mode: MessageMode = "info") {
    message.value = text;
    messageMode.value = mode;
  }

  function addSources(paths: string[], kind: "file" | "folder") {
    const existing = new Set(sources.value.map((s) => s.path));
    for (const path of paths) {
      if (existing.has(path)) continue;
      sources.value.push({ path, name: baseName(path), kind });
      existing.add(path);
    }
  }

  // Kéo-thả từ ngoài vào (Tauri webview drag-drop) chỉ cho path thô, không kèm biết file hay
  // folder — không có lệnh "stat" riêng nên tận dụng explorer_read_dir đã có sẵn: đọc được ⇒ folder,
  // lỗi ⇒ file. Giữ đúng thứ tự thả (không tách nhóm file/folder như 2 nút "Thêm file"/"Thêm folder").
  async function classifyPath(path: string): Promise<"file" | "folder"> {
    try {
      await explorerReadDir(path);
      return "folder";
    } catch {
      return "file";
    }
  }

  async function addDroppedPaths(paths: string[]) {
    const existing = new Set(sources.value.map((s) => s.path));
    const uniquePaths = paths.filter((p) => !existing.has(p));
    if (!uniquePaths.length) return;
    const kinds = await Promise.all(uniquePaths.map(classifyPath));
    uniquePaths.forEach((path, i) => {
      sources.value.push({ path, name: baseName(path), kind: kinds[i] });
    });
  }

  async function pickFiles() {
    if (!canUseTauriRuntime()) return setMessage(tauriRuntimeMessage, "error");
    try {
      const selected = await open({ multiple: true, directory: false });
      if (Array.isArray(selected)) addSources(selected, "file");
      else if (typeof selected === "string") addSources([selected], "file");
    } catch (e) {
      setMessage(friendlyError(e), "error");
    }
  }

  async function pickFolders() {
    if (!canUseTauriRuntime()) return setMessage(tauriRuntimeMessage, "error");
    try {
      const selected = await open({ multiple: true, directory: true });
      if (Array.isArray(selected)) addSources(selected, "folder");
      else if (typeof selected === "string") addSources([selected], "folder");
    } catch (e) {
      setMessage(friendlyError(e), "error");
    }
  }

  async function pickOutputDir() {
    if (!canUseTauriRuntime()) return setMessage(tauriRuntimeMessage, "error");
    try {
      const selected = await open({ multiple: false, directory: true });
      if (typeof selected === "string") config.outputDir = selected;
    } catch (e) {
      setMessage(friendlyError(e), "error");
    }
  }

  function removeSource(path: string) {
    sources.value = sources.value.filter((s) => s.path !== path);
  }

  function clearSources() {
    sources.value = [];
    parts.value = [];
    showResult.value = false;
    resultMode.value = null;
    previewArchives.value = [];
    previewSizeBytes.value = null;
  }

  function reset() {
    clearSources();
    config.outputDir = "";
    config.archiveName = "attachment";
    config.limitMb = null;
    config.password = "";
    config.noSplit = true;
    setMessage("Đã đặt lại thông số.");
  }

  async function run() {
    if (!canRun.value) {
      setMessage("Cần ít nhất 1 nguồn.", "error");
      return;
    }
    if (!canUseTauriRuntime()) {
      setMessage(tauriRuntimeMessage, "error");
      return;
    }
    if (!config.outputDir.trim()) {
      setMessage("Vui lòng chọn thư mục xuất.", "error");
      return;
    }

    running.value = true;
    setMessage("Đang nén và tách file...");
    try {
      const result = await fileSplitRun({
        sources: sources.value.map((s) => ({ path: s.path, name: s.name })),
        outputDir: config.outputDir.trim(),
        archiveName: config.archiveName,
        limitMb: config.noSplit ? null : config.limitMb,
        password: config.password,
      });
      previewArchives.value = [];
      previewSizeBytes.value = null;
      parts.value = result.files;
      resultMode.value = "run";
      showResult.value = true;

      const mb = (result.totalBytes / (1024 * 1024)).toFixed(1);
      const enc = result.encrypted ? " · AES-256" : "";
      const splitNote =
        result.splitCount > 0
          ? ` · đã cắt thành ${result.files.length} phần`
          : " · không cần tách (dưới giới hạn)";
      setMessage(`Tạo ${result.files.length} file (${mb} MB)${enc}${splitNote}.`);
    } catch (e) {
      setMessage(friendlyError(e), "error");
    } finally {
      running.value = false;
    }
  }

  return {
    sources,
    config,
    message,
    messageMode,
    running,
    parts,
    showResult,
    resultMode,
    previewArchives,
    previewSizeBytes,
    canRun,
    pickFiles,
    pickFolders,
    addDroppedPaths,
    pickOutputDir,
    removeSource,
    clearSources,
    reset,
    run,
    preview,
  };
}
