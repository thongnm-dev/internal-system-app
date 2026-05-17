import { FileInput, FolderOpen } from "lucide-react";
import { Button } from "primereact/button";
import { Column } from "primereact/column";
import { DataTable } from "primereact/datatable";
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

        <DataTable
          className="app-data-table min-h-0"
          emptyMessage="No imported issues. Select a project and CSV file, then click Import."
          scrollable
          scrollHeight="flex"
          tableStyle={{ minWidth: "1680px" }}
          value={rows}
        >
          <Column field="subject" header="Subject" bodyClassName="max-w-[280px] truncate font-bold text-ink" />
          <Column field="description" header="Description" bodyClassName="max-w-[320px] truncate" />
          <Column field="issueType" header="Issue Type" bodyClassName="whitespace-nowrap" />
          <Column field="assignee" header="Assignee" bodyClassName="whitespace-nowrap" />
          <Column field="startDate" header="Start Date" bodyClassName="whitespace-nowrap" />
          <Column field="dueDate" header="Due Date" bodyClassName="whitespace-nowrap" />
          <Column header="Estimated Hours" body={(row: IssueCsvRow) => formatEmpty(row.estimatedHours)} bodyClassName="num" headerClassName="num" />
          <Column header="Actual Hours" body={(row: IssueCsvRow) => formatEmpty(row.actualHours)} bodyClassName="num" headerClassName="num" />
          <Column field="categories" header="Categories" bodyClassName="max-w-[220px] truncate" />
          <Column field="version" header="Version" bodyClassName="whitespace-nowrap" />
          <Column field="milestones" header="Milestones" bodyClassName="whitespace-nowrap" />
          <Column field="priority" header="Priority" body={priorityBody} bodyClassName="whitespace-nowrap" />
          <Column field="parentIssue" header="Parent issue" bodyClassName="whitespace-nowrap" />
          <Column field="bugTypes" header="Bug Types" bodyClassName="whitespace-nowrap" />
          <Column field="bugSeverityLevels" header="Bug severity levels" bodyClassName="whitespace-nowrap" />
          <Column field="testPhase" header="Test Phase" bodyClassName="whitespace-nowrap" />
        </DataTable>
      </section>
    </section>
  );
}

function priorityBody(row: IssueCsvRow) {
  return (
    row.priority ? <span className={["rounded px-2 py-1 text-xs font-bold", priorityTone(row.priority)].join(" ")}>{row.priority}</span> : ""
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
