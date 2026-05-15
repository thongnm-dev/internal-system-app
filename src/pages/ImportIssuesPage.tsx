import { FileInput, FolderOpen } from "lucide-react";
import { Button } from "primereact/button";
import { Dropdown } from "primereact/dropdown";
import { useMemo, useRef, useState } from "react";
import { MessageBanner } from "../components/MessageBanner";
import type { MessageMode } from "../types/statistics";

type IssueCsvRow = {
  actualHours: string;
  assignee: string;
  bugSeverityLevels: string;
  bugTypes: string;
  categories: string;
  description: string;
  dueDate: string;
  estimatedHours: string;
  issueType: string;
  milestones: string;
  parentIssue: string;
  priority: string;
  startDate: string;
  subject: string;
  testPhase: string;
  version: string;
};

const projects = [
  { label: "YUJI - PJ Yuji Internal Tool", value: "YUJI" },
  { label: "HRP - HR Portal", value: "HRP" },
  { label: "SALE - Sales Dashboard", value: "SALE" },
  { label: "MOB - Mobile Companion App", value: "MOB" },
  { label: "OPS - Infrastructure Support", value: "OPS" },
];

const issueCsvHeaders = [
  "Subject",
  "Description",
  "Issue Type",
  "Assignee",
  "Start Date",
  "Due Date",
  "Estimated Hours",
  "Actual Hours",
  "Categories",
  "Version",
  "Milestones",
  "Priority",
  "Parent issue",
  "Bug Types",
  "Bug severity levels",
  "Test Phase",
] as const;

const emptyImportMessage = "Select a project and a CSV file, then click Import.";

export function ImportIssuesPage() {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [projectCode, setProjectCode] = useState("");
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [rows, setRows] = useState<IssueCsvRow[]>([]);
  const [message, setMessage] = useState(emptyImportMessage);
  const [messageMode, setMessageMode] = useState<MessageMode>("info");
  const [isImporting, setIsImporting] = useState(false);

  const selectedProject = useMemo(
    () => projects.find((project) => project.value === projectCode),
    [projectCode],
  );

  const importCsv = async () => {
    if (!projectCode) {
      setMessage("Please select a project before importing issues.");
      setMessageMode("error");
      return;
    }

    if (!selectedFile) {
      setMessage("Please select an issue CSV file before importing.");
      setMessageMode("error");
      return;
    }

    setIsImporting(true);
    setMessage("Importing issue CSV...");
    setMessageMode("info");

    try {
      const content = await selectedFile.text();
      const result = parseIssueCsv(content);
      setRows(result);
      setMessage(
        `Imported ${result.length.toLocaleString("en-US")} issues for ${selectedProject?.label ?? projectCode}.`,
      );
      setMessageMode("info");
    } catch (error) {
      setRows([]);
      setMessage(error instanceof Error ? error.message : "Could not import the selected issue CSV file.");
      setMessageMode("error");
    } finally {
      setIsImporting(false);
    }
  };

  const selectFile = (file: File | null) => {
    setSelectedFile(file);
    setRows([]);
    setMessage(file ? `${file.name} selected. Click Import to preview issues.` : emptyImportMessage);
    setMessageMode("info");
  };

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <section className="rounded-lg border border-stone-200 bg-white p-4 shadow-sm">
        <div className="flex items-center gap-2">
          <FileInput className="h-5 w-5 text-brand" />
          <h3 className="font-bold">Import issues</h3>
        </div>

        <div className="mt-4 grid gap-3 lg:grid-cols-[minmax(220px,320px)_minmax(0,1fr)_auto_auto]">
          <label className="block min-w-0">
            <span className="text-xs font-bold text-slate-500">Project</span>
            <Dropdown
              className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              options={projects}
              placeholder="Select project"
              value={projectCode}
              onChange={(event) => setProjectCode(event.value ?? "")}
            />
          </label>

          <label className="block min-w-0">
            <span className="text-xs font-bold text-slate-500">CSV file</span>
            <div className="mt-1 flex h-10 items-center rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-700">
              <span className="min-w-0 truncate">{selectedFile?.name ?? "Select issues CSV file..."}</span>
            </div>
          </label>

          <div className="flex items-end">
            <input
              ref={fileInputRef}
              className="hidden"
              type="file"
              accept=".csv,text/csv"
              onChange={(event) => selectFile(event.target.files?.[0] ?? null)}
            />
            <Button
              className="flex h-10 w-full items-center justify-center gap-2 rounded-md border border-slate-300 bg-white px-4 text-sm font-bold text-slate-700 hover:bg-slate-50"
              type="button"
              title="Browse CSV"
              onClick={() => fileInputRef.current?.click()}
            >
              <FolderOpen className="h-4 w-4" />
              Browse
            </Button>
          </div>

          <div className="flex items-end">
            <Button
              className="flex h-10 w-full items-center justify-center gap-2 rounded-md bg-slate-800 px-4 text-sm font-bold text-white hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-60"
              type="button"
              disabled={isImporting}
              onClick={() => void importCsv()}
            >
              <FileInput className="h-4 w-4" />
              Import
            </Button>
          </div>
        </div>
      </section>

      <MessageBanner message={message} mode={messageMode} />

      <section className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-stone-200 bg-panel shadow-sm">
        <div className="flex items-center justify-between gap-4 border-b border-stone-200 px-4 py-3">
          <div className="min-w-0">
            <h3 className="font-bold">Imported issue list</h3>
            <p className="mt-1 truncate text-xs text-slate-500">
              {selectedProject ? selectedProject.label : "No project selected"}
              {selectedFile ? ` / ${selectedFile.name}` : ""}
            </p>
          </div>
          <span className="text-xs text-slate-500">{rows.length.toLocaleString("en-US")} issues</span>
        </div>

        <div className="min-h-0 overflow-auto">
          <table className="w-full min-w-[1680px] border-collapse">
            <thead>
              <tr>
                <th className="table-head">Subject</th>
                <th className="table-head">Description</th>
                <th className="table-head">Issue Type</th>
                <th className="table-head">Assignee</th>
                <th className="table-head">Start Date</th>
                <th className="table-head">Due Date</th>
                <th className="table-head num">Estimated Hours</th>
                <th className="table-head num">Actual Hours</th>
                <th className="table-head">Categories</th>
                <th className="table-head">Version</th>
                <th className="table-head">Milestones</th>
                <th className="table-head">Priority</th>
                <th className="table-head">Parent issue</th>
                <th className="table-head">Bug Types</th>
                <th className="table-head">Bug severity levels</th>
                <th className="table-head">Test Phase</th>
              </tr>
            </thead>
            <tbody>
              {rows.length === 0 ? (
                <tr>
                  <td className="table-cell h-48 text-center text-slate-500" colSpan={issueCsvHeaders.length}>
                    No imported issues. Select a project and CSV file, then click Import.
                  </td>
                </tr>
              ) : (
                rows.map((row, index) => <IssueImportRow key={`${row.subject}-${index}`} row={row} />)
              )}
            </tbody>
          </table>
        </div>
      </section>
    </section>
  );
}

function IssueImportRow({ row }: { row: IssueCsvRow }) {
  return (
    <tr className="hover:bg-slate-50">
      <td className="table-cell max-w-[280px] truncate font-bold text-ink">{row.subject}</td>
      <td className="table-cell max-w-[320px] truncate">{row.description}</td>
      <td className="table-cell whitespace-nowrap">{row.issueType}</td>
      <td className="table-cell whitespace-nowrap">{row.assignee}</td>
      <td className="table-cell whitespace-nowrap">{row.startDate}</td>
      <td className="table-cell whitespace-nowrap">{row.dueDate}</td>
      <td className="table-cell num">{formatEmpty(row.estimatedHours)}</td>
      <td className="table-cell num">{formatEmpty(row.actualHours)}</td>
      <td className="table-cell max-w-[220px] truncate">{row.categories}</td>
      <td className="table-cell whitespace-nowrap">{row.version}</td>
      <td className="table-cell whitespace-nowrap">{row.milestones}</td>
      <td className="table-cell whitespace-nowrap">
        {row.priority ? <span className={["rounded px-2 py-1 text-xs font-bold", priorityTone(row.priority)].join(" ")}>{row.priority}</span> : ""}
      </td>
      <td className="table-cell whitespace-nowrap">{row.parentIssue}</td>
      <td className="table-cell whitespace-nowrap">{row.bugTypes}</td>
      <td className="table-cell whitespace-nowrap">{row.bugSeverityLevels}</td>
      <td className="table-cell whitespace-nowrap">{row.testPhase}</td>
    </tr>
  );
}

function parseIssueCsv(content: string): IssueCsvRow[] {
  const records = parseCsvRecords(content);
  const [headers, ...dataRows] = records.filter((record) => record.some((cell) => cell.trim()));

  if (!headers) {
    return [];
  }

  const normalizedHeaders = headers.map((header) => header.trim());
  const hasExpectedHeaders = issueCsvHeaders.every((header, index) => normalizedHeaders[index] === header);
  if (!hasExpectedHeaders) {
    throw new Error("CSV headers do not match the issue import template.");
  }

  return dataRows.map((record) => ({
    subject: readCell(record, 0),
    description: readCell(record, 1),
    issueType: readCell(record, 2),
    assignee: readCell(record, 3),
    startDate: readCell(record, 4),
    dueDate: readCell(record, 5),
    estimatedHours: readCell(record, 6),
    actualHours: readCell(record, 7),
    categories: readCell(record, 8),
    version: readCell(record, 9),
    milestones: readCell(record, 10),
    priority: readCell(record, 11),
    parentIssue: readCell(record, 12),
    bugTypes: readCell(record, 13),
    bugSeverityLevels: readCell(record, 14),
    testPhase: readCell(record, 15),
  }));
}

function parseCsvRecords(content: string) {
  const records: string[][] = [];
  let cell = "";
  let record: string[] = [];
  let isQuoted = false;

  for (let index = 0; index < content.length; index += 1) {
    const char = content[index];
    const nextChar = content[index + 1];

    if (char === '"') {
      if (isQuoted && nextChar === '"') {
        cell += '"';
        index += 1;
      } else {
        isQuoted = !isQuoted;
      }
      continue;
    }

    if (char === "," && !isQuoted) {
      record.push(cell);
      cell = "";
      continue;
    }

    if ((char === "\n" || char === "\r") && !isQuoted) {
      if (char === "\r" && nextChar === "\n") {
        index += 1;
      }
      record.push(cell);
      records.push(record);
      record = [];
      cell = "";
      continue;
    }

    cell += char;
  }

  record.push(cell);
  records.push(record);
  return records;
}

function readCell(record: string[], index: number) {
  return (record[index] ?? "").trim();
}

function formatEmpty(value: string) {
  return value || "-";
}

function priorityTone(priority: string) {
  if (priority === "High") {
    return "bg-orange-100 text-orange-800";
  }

  if (priority === "Normal") {
    return "bg-sky-100 text-sky-800";
  }

  if (priority === "Low") {
    return "bg-slate-100 text-slate-700";
  }

  return "bg-red-100 text-red-800";
}
