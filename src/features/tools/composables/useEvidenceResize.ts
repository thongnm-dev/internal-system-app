import { open, save } from "@tauri-apps/plugin-dialog";
import { ref } from "vue";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { resizeEvidenceImages } from "@/tauri/commands/evidence-resize";
import type { MessageMode } from "@/_/types/app";
import type { EvidenceResizeOptions, EvidenceResizeResult } from "@/_/types/evidence-resize";

function defaultResizedPath(path: string) {
  if (!path.trim()) return "";
  return path.replace(/(\.[^.\\/]+)?$/i, (ext) => `_resized${ext || ".xlsx"}`);
}

export function useEvidenceResize() {
  const inputPath = ref("");
  const outputPath = ref("");
  const widthCm = ref<number | null>(null);
  const heightCm = ref<number | null>(null);
  const pageBreakPreview = ref(false);
  const zoomEnabled = ref(false);
  const zoomPercent = ref<number | null>(100);
  const startColumnEnabled = ref(false);
  const startColumn = ref("");
  const fontName = ref("");
  const fontSize = ref<number | null>(null);
  const avoidCoveringContent = ref(true);
  const result = ref<EvidenceResizeResult | null>(null);
  const message = ref("Select an Excel workbook, optionally set Width/Height, then resize.");
  const messageMode = ref<MessageMode>("info");
  const isProcessing = ref(false);

  function updateInputPath(value: string) {
    inputPath.value = value;
    if (!outputPath.value) outputPath.value = defaultResizedPath(value);
    result.value = null;
  }

  function setOutputPath(value: string) {
    outputPath.value = value;
  }

  async function pickInputFile() {
    if (!canUseTauriRuntime()) {
      message.value = tauriRuntimeMessage;
      messageMode.value = "error";
      return;
    }
    try {
      const selected = await open({ multiple: false, filters: [{ name: "Excel workbook", extensions: ["xlsx"] }] });
      if (typeof selected === "string") {
        inputPath.value = selected;
        outputPath.value = defaultResizedPath(selected);
        result.value = null;
        message.value = "Excel file selected. Confirm the output path and sizes, then resize.";
        messageMode.value = "info";
      }
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
    }
  }

  async function pickOutputFile() {
    if (!canUseTauriRuntime()) {
      message.value = tauriRuntimeMessage;
      messageMode.value = "error";
      return;
    }
    try {
      const selected = await save({
        defaultPath: outputPath.value || defaultResizedPath(inputPath.value),
        filters: [{ name: "Excel workbook", extensions: ["xlsx"] }],
      });
      if (typeof selected === "string") {
        outputPath.value = selected;
        message.value = "Output path selected.";
        messageMode.value = "info";
      }
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
    }
  }

  async function run() {
    if (!inputPath.value.trim()) {
      message.value = "Please select an Excel workbook before resizing.";
      messageMode.value = "error";
      return;
    }
    if (!outputPath.value.trim()) {
      message.value = "Please choose where to save the resized workbook.";
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

    const options: EvidenceResizeOptions = {
      pageBreakPreview: pageBreakPreview.value,
      zoomPercent: zoomEnabled.value ? zoomPercent.value : null,
      startColumn: startColumnEnabled.value ? startColumn.value.trim() : null,
      fontName: fontName.value.trim() ? fontName.value.trim() : null,
      fontSize: fontSize.value,
      avoidCoveringContent: avoidCoveringContent.value,
    };

    isProcessing.value = true;
    message.value = "Resizing evidence images...";
    messageMode.value = "info";
    try {
      const resized = await resizeEvidenceImages(
        inputPath.value,
        outputPath.value,
        widthCm.value && widthCm.value > 0 ? widthCm.value : null,
        heightCm.value && heightCm.value > 0 ? heightCm.value : null,
        options,
      );
      result.value = resized;
      message.value = `${resized.images_resized} image(s) resized across ${resized.drawings_processed} sheet(s): ${resized.output_file_name}`;
      messageMode.value = "info";
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
    } finally {
      isProcessing.value = false;
    }
  }

  return {
    inputPath,
    outputPath,
    widthCm,
    heightCm,
    pageBreakPreview,
    zoomEnabled,
    zoomPercent,
    startColumnEnabled,
    startColumn,
    fontName,
    fontSize,
    avoidCoveringContent,
    result,
    message,
    messageMode,
    isProcessing,
    updateInputPath,
    setOutputPath,
    pickInputFile,
    pickOutputFile,
    run,
  };
}
