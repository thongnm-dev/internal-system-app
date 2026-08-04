import { open } from "@tauri-apps/plugin-dialog";
import { ref } from "vue";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { resizeExcelImages } from "@/tauri/commands/excel-helper";
import type { MessageMode } from "@/_/types/app";
import type { ExcelHelperOptions, ExcelHelperResult } from "@/_/types/excel-helper";

export function fileNameFromPath(path: string) {
  return path.split(/[\\/]/).pop() || path;
}

function baseNameWithoutExt(fileName: string) {
  return fileName.replace(/\.[^./\\]+$/, "");
}

function joinPath(dir: string, file: string) {
  return `${dir.replace(/[\\/]+$/, "")}/${file}`;
}

function isXlsxPath(path: string) {
  return /\.xlsx$/i.test(path);
}

const DEFAULT_MESSAGE = "Drag & drop one or more .xlsx files, choose an output folder, then resize.";

export function useExcelHelper() {
  const inputPaths = ref<string[]>([]);
  const outputFolder = ref("");
  const widthCm = ref<number | null>(null);
  const heightCm = ref<number | null>(null);
  const pageBreakPreview = ref(false);
  const zoomEnabled = ref(false);
  const zoomPercent = ref<number | null>(100);
  const startColumnEnabled = ref(false);
  const startColumn = ref("");
  const fontName = ref("");
  const fontSize = ref<number | null>(null);
  const resetActiveCell = ref(false);
  const avoidCoveringContent = ref(true);
  const results = ref<ExcelHelperResult[]>([]);
  const message = ref(DEFAULT_MESSAGE);
  const messageMode = ref<MessageMode>("info");
  const isProcessing = ref(false);
  const processedCount = ref(0);

  function reset() {
    inputPaths.value = [];
    outputFolder.value = "";
    widthCm.value = null;
    heightCm.value = null;
    pageBreakPreview.value = false;
    zoomEnabled.value = false;
    zoomPercent.value = 100;
    startColumnEnabled.value = false;
    startColumn.value = "";
    fontName.value = "";
    fontSize.value = null;
    resetActiveCell.value = false;
    avoidCoveringContent.value = true;
    results.value = [];
    processedCount.value = 0;
    message.value = DEFAULT_MESSAGE;
    messageMode.value = "info";
  }

  function addFiles(paths: string[]) {
    const valid = paths.filter(isXlsxPath);
    for (const p of valid) {
      if (!inputPaths.value.includes(p)) inputPaths.value.push(p);
    }
    results.value = [];
    if (valid.length) {
      message.value = `${inputPaths.value.length} file(s) ready to resize.`;
      messageMode.value = "info";
    } else if (paths.length) {
      message.value = "Only .xlsx files are supported.";
      messageMode.value = "error";
    }
  }

  function removeFile(path: string) {
    inputPaths.value = inputPaths.value.filter((p) => p !== path);
    results.value = results.value.filter((r) => r.source_path !== path);
  }

  function clearFiles() {
    inputPaths.value = [];
    results.value = [];
  }

  function setOutputFolder(value: string) {
    outputFolder.value = value;
  }

  async function pickInputFiles() {
    if (!canUseTauriRuntime()) {
      message.value = tauriRuntimeMessage;
      messageMode.value = "error";
      return;
    }
    try {
      const selected = await open({ multiple: true, filters: [{ name: "Excel workbook", extensions: ["xlsx"] }] });
      if (Array.isArray(selected) && selected.length) addFiles(selected);
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
    }
  }

  async function pickOutputFolder() {
    if (!canUseTauriRuntime()) {
      message.value = tauriRuntimeMessage;
      messageMode.value = "error";
      return;
    }
    try {
      const selected = await open({ directory: true });
      if (typeof selected === "string") {
        outputFolder.value = selected;
        message.value = "Output folder selected.";
        messageMode.value = "info";
      }
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
    }
  }

  async function run() {
    if (!inputPaths.value.length) {
      message.value = "Please add at least one Excel workbook before resizing.";
      messageMode.value = "error";
      return;
    }
    if (!outputFolder.value.trim()) {
      message.value = "Please choose an output folder.";
      messageMode.value = "error";
      return;
    }
    if (zoomEnabled.value && (!zoomPercent.value || zoomPercent.value < 10 || zoomPercent.value > 400)) {
      message.value = "Please enter a zoom level between 10% and 400%.";
      messageMode.value = "error";
      return;
    }
    if (startColumnEnabled.value && !startColumn.value.trim()) {
      message.value = "Please enter a start column, e.g. B or B2.";
      messageMode.value = "error";
      return;
    }
    if (fontSize.value !== null && fontSize.value <= 0) {
      message.value = "Please enter a font size greater than 0.";
      messageMode.value = "error";
      return;
    }

    const options: ExcelHelperOptions = {
      pageBreakPreview: pageBreakPreview.value,
      zoomPercent: zoomEnabled.value ? zoomPercent.value : null,
      startColumn: startColumnEnabled.value ? startColumn.value.trim() : null,
      fontName: fontName.value.trim() ? fontName.value.trim() : null,
      fontSize: fontSize.value,
      resetActiveCell: resetActiveCell.value,
      avoidCoveringContent: avoidCoveringContent.value,
    };

    isProcessing.value = true;
    processedCount.value = 0;
    results.value = [];
    const total = inputPaths.value.length;
    message.value = `Resizing evidence images (0/${total})...`;
    messageMode.value = "info";

    const failures: string[] = [];
    for (const inputPath of inputPaths.value) {
      const outputPath = joinPath(outputFolder.value, `${baseNameWithoutExt(fileNameFromPath(inputPath))}_resized.xlsx`);
      try {
        const resized = await resizeExcelImages(
          inputPath,
          outputPath,
          widthCm.value && widthCm.value > 0 ? widthCm.value : null,
          heightCm.value && heightCm.value > 0 ? heightCm.value : null,
          options,
          null,
        );
        results.value.push(resized);
      } catch (e) {
        failures.push(`${fileNameFromPath(inputPath)}: ${friendlyError(e)}`);
      }
      processedCount.value += 1;
      message.value = `Resizing evidence images (${processedCount.value}/${total})...`;
    }

    isProcessing.value = false;
    if (failures.length) {
      message.value = `${results.value.length} succeeded, ${failures.length} failed — ${failures.join("; ")}`;
      messageMode.value = "error";
    } else {
      const totalImages = results.value.reduce((sum, r) => sum + r.images_resized, 0);
      message.value = `${results.value.length} file(s) resized, ${totalImages} image(s) total.`;
      messageMode.value = "info";
    }
  }

  return {
    inputPaths,
    outputFolder,
    widthCm,
    heightCm,
    pageBreakPreview,
    zoomEnabled,
    zoomPercent,
    startColumnEnabled,
    startColumn,
    fontName,
    fontSize,
    resetActiveCell,
    avoidCoveringContent,
    results,
    message,
    messageMode,
    isProcessing,
    processedCount,
    addFiles,
    removeFile,
    clearFiles,
    setOutputFolder,
    pickInputFiles,
    pickOutputFolder,
    run,
    reset,
  };
}
