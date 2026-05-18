import { CheckCircle2, FileCode2, FileSpreadsheet, FolderOpen, Save, Wand2 } from "lucide-react";
import { Button } from "primereact/button";
import { InputText } from "primereact/inputtext";
import { MessageBanner } from "../components/MessageBanner";
import type { MessageMode, XlsxMarkdownResult } from "../types/statistics";

type XlsxMarkdownPageProps = {
  inputPath: string;
  isConverting: boolean;
  message: string;
  messageMode: MessageMode;
  outputPath: string;
  result: XlsxMarkdownResult | null;
  onConvert: () => void;
  onInputPathChange: (value: string) => void;
  onOutputPathChange: (value: string) => void;
  onPickInputFile: () => void;
  onPickOutputFile: () => void;
};

export function XlsxMarkdownPage({
  inputPath,
  isConverting,
  message,
  messageMode,
  outputPath,
  result,
  onConvert,
  onInputPathChange,
  onOutputPathChange,
  onPickInputFile,
  onPickOutputFile,
}: XlsxMarkdownPageProps) {
  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <div className="grid grid-cols-[minmax(0,1fr)_320px] gap-4">
        <section className="rounded-lg border border-stone-200 bg-white p-4 shadow-sm">
          <div className="flex items-center gap-2">
            <FileSpreadsheet className="h-5 w-5 text-brand" />
            <h3 className="font-bold">Excel workbook</h3>
          </div>

          <div className="mt-4 grid gap-3">
            <label className="grid gap-1.5">
              <span className="text-xs font-bold uppercase tracking-wide text-slate-500">Input .xlsx</span>
              <div className="grid grid-cols-[minmax(0,1fr)_auto] gap-2">
                <InputText
                  className="h-10 min-w-0 rounded-md border border-slate-300 bg-white px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  placeholder="Select Excel workbook..."
                  type="text"
                  value={inputPath}
                  onChange={(event) => onInputPathChange(event.target.value)}
                />
                <Button
                  className="flex h-10 w-10 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-700 hover:bg-slate-50"
                  type="button"
                  title="Browse Excel workbook"
                  onClick={onPickInputFile}
                >
                  <FolderOpen className="h-4 w-4" />
                </Button>
              </div>
            </label>

            <label className="grid gap-1.5">
              <span className="text-xs font-bold uppercase tracking-wide text-slate-500">Output .md</span>
              <div className="grid grid-cols-[minmax(0,1fr)_auto] gap-2">
                <InputText
                  className="h-10 min-w-0 rounded-md border border-slate-300 bg-white px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  placeholder="Markdown output path..."
                  type="text"
                  value={outputPath}
                  onChange={(event) => onOutputPathChange(event.target.value)}
                />
                <Button
                  className="flex h-10 w-10 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-700 hover:bg-slate-50"
                  type="button"
                  title="Choose Markdown output"
                  onClick={onPickOutputFile}
                >
                  <Save className="h-4 w-4" />
                </Button>
              </div>
            </label>

            <Button
              className="inline-flex h-10 w-fit items-center justify-center gap-2 rounded-md bg-slate-800 px-4 text-sm font-bold text-white hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-60"
              type="button"
              disabled={isConverting}
              onClick={onConvert}
            >
              <Wand2 className="h-4 w-4" />
              {isConverting ? "Converting..." : "Convert"}
            </Button>
          </div>
        </section>

        <aside className="rounded-lg border border-stone-200 bg-white p-4 shadow-sm">
          <span className="text-sm font-bold text-slate-500">Last conversion</span>
          <strong className="mt-2 block truncate text-lg leading-tight text-slate-900">
            {result?.output_file_name ?? "-"}
          </strong>
          <p className="mt-2 text-xs text-slate-500">{result ? result.output_path : "No Markdown file created yet."}</p>
          {result && (
            <div className="mt-4 flex items-center gap-2 text-sm font-bold text-brand">
              <CheckCircle2 className="h-4 w-4" />
              Ready
            </div>
          )}
        </aside>
      </div>

      <MessageBanner message={message} mode={messageMode} />

      <section className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-stone-200 bg-white shadow-sm">
        <div className="flex items-center justify-between gap-3 border-b border-stone-200 px-4 py-3">
          <div className="flex min-w-0 items-center gap-2">
            <FileCode2 className="h-4 w-4 text-brand" />
            <h3 className="font-bold">Markdown preview</h3>
          </div>
          <span className="truncate text-xs font-semibold text-slate-500">{result?.source_file_name ?? "No file selected"}</span>
        </div>

        <pre className="min-h-0 flex-1 overflow-auto whitespace-pre-wrap break-words bg-slate-950 p-4 text-sm leading-6 text-slate-100">
          {result?.markdown || "Converted Markdown will appear here."}
        </pre>
      </section>
    </section>
  );
}
