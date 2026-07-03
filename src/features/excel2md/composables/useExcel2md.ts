import { open, save } from "@tauri-apps/plugin-dialog";
import { ref } from "vue";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";
import { canUseTauriRuntime, convertXlsxSpecToMarkdown, friendlyError } from "@/tauri/commands";
import type { MessageMode, XlsxMarkdownResult } from "@/shared/types/statistics";

function defaultMarkdownPath(path: string) {
  return path.trim() ? path.replace(/\.[^.\\/]+$/i, ".md") : "";
}

export function useExcel2md() {
  const inputPath = ref("");
  const outputPath = ref("");
  const result = ref<XlsxMarkdownResult | null>(null);
  const message = ref("Select an Excel workbook, then convert it to Markdown.");
  const messageMode = ref<MessageMode>("info");
  const isConverting = ref(false);

  function updateInputPath(value: string) {
    inputPath.value = value;
    if (!outputPath.value) outputPath.value = defaultMarkdownPath(value);
    result.value = null;
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
        outputPath.value = defaultMarkdownPath(selected);
        result.value = null;
        message.value = "Excel file selected. Confirm the Markdown output path, then convert.";
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
        defaultPath: outputPath.value || defaultMarkdownPath(inputPath.value),
        filters: [{ name: "Markdown", extensions: ["md"] }],
      });
      if (typeof selected === "string") {
        outputPath.value = selected;
        message.value = "Markdown output path selected.";
        messageMode.value = "info";
      }
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
    }
  }

  async function convert() {
    if (!inputPath.value.trim()) {
      message.value = "Please select an Excel workbook before converting.";
      messageMode.value = "error";
      return;
    }
    isConverting.value = true;
    message.value = "Converting Excel workbook to Markdown...";
    messageMode.value = "info";
    try {
      const conversion = await convertXlsxSpecToMarkdown(inputPath.value, outputPath.value.trim() || null);
      result.value = conversion;
      outputPath.value = conversion.output_path;
      message.value = `Markdown created: ${conversion.output_file_name}`;
      messageMode.value = "info";
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
    } finally {
      isConverting.value = false;
    }
  }

  return { inputPath, outputPath, result, message, messageMode, isConverting, updateInputPath, pickInputFile, pickOutputFile, convert, setOutputPath: (v: string) => { outputPath.value = v; } };
}
