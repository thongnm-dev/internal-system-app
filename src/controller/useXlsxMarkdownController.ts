import { open, save } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { tauriRuntimeMessage } from "../config/appConfig";
import { canUseTauriRuntime, friendlyError, safeInvoke } from "../core/tauriRuntime";
import type { MessageMode, XlsxMarkdownResult } from "../types/statistics";

export function useXlsxMarkdownController() {
  const [inputPath, setInputPath] = useState("");
  const [outputPath, setOutputPath] = useState("");
  const [result, setResult] = useState<XlsxMarkdownResult | null>(null);
  const [message, setMessage] = useState("Select an Excel workbook, then convert it to Markdown.");
  const [messageMode, setMessageMode] = useState<MessageMode>("info");
  const [isConverting, setIsConverting] = useState(false);

  const updateInputPath = (value: string) => {
    setInputPath(value);
    setOutputPath((current) => current || defaultMarkdownPath(value));
    setResult(null);
  };

  const pickInputFile = async () => {
    if (!canUseTauriRuntime()) {
      setMessage(tauriRuntimeMessage);
      setMessageMode("error");
      return;
    }

    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "Excel workbook", extensions: ["xlsx"] }],
      });

      if (typeof selected === "string") {
        setInputPath(selected);
        setOutputPath(defaultMarkdownPath(selected));
        setResult(null);
        setMessage("Excel file selected. Confirm the Markdown output path, then convert.");
        setMessageMode("info");
      }
    } catch (error) {
      setMessage(friendlyError(error));
      setMessageMode("error");
    }
  };

  const pickOutputFile = async () => {
    if (!canUseTauriRuntime()) {
      setMessage(tauriRuntimeMessage);
      setMessageMode("error");
      return;
    }

    try {
      const selected = await save({
        defaultPath: outputPath || defaultMarkdownPath(inputPath),
        filters: [{ name: "Markdown", extensions: ["md"] }],
      });

      if (typeof selected === "string") {
        setOutputPath(selected);
        setMessage("Markdown output path selected.");
        setMessageMode("info");
      }
    } catch (error) {
      setMessage(friendlyError(error));
      setMessageMode("error");
    }
  };

  const convert = async () => {
    if (!inputPath.trim()) {
      setMessage("Please select an Excel workbook before converting.");
      setMessageMode("error");
      return;
    }

    setIsConverting(true);
    setMessage("Converting Excel workbook to Markdown...");
    setMessageMode("info");

    try {
      const conversion = await safeInvoke<XlsxMarkdownResult>("convert_xlsx_spec_to_markdown", {
        inputPath,
        outputPath: outputPath.trim() || null,
      });
      setResult(conversion);
      setOutputPath(conversion.output_path);
      setMessage(`Markdown created: ${conversion.output_file_name}`);
      setMessageMode("info");
    } catch (error) {
      setMessage(friendlyError(error));
      setMessageMode("error");
    } finally {
      setIsConverting(false);
    }
  };

  return {
    convert,
    inputPath,
    isConverting,
    message,
    messageMode,
    outputPath,
    pickInputFile,
    pickOutputFile,
    result,
    setInputPath: updateInputPath,
    setOutputPath,
  };
}

function defaultMarkdownPath(path: string) {
  if (!path.trim()) {
    return "";
  }

  return path.replace(/\.[^.\\/]+$/i, ".md");
}
