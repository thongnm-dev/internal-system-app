import { computed, ref, watch } from "vue";
import type { MessageMode } from "@/shared/types/statistics";

export type IssueCsvRow = {
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

export type ProjectOption = { label: string; value: string };

const defaultProjects: ProjectOption[] = [
  { label: "Billing Portal", value: "Billing Portal" },
  { label: "Internal Extension", value: "Internal Extension" },
  { label: "Mobile Gateway", value: "Mobile Gateway" },
  { label: "Reporting Hub", value: "Reporting Hub" },
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

export function useImportIssues(initialProject = "") {
  const projectCode = ref(initialProject);
  const selectedFile = ref<File | null>(null);
  const rows = ref<IssueCsvRow[]>([]);
  const message = ref(emptyImportMessage);
  const messageMode = ref<MessageMode>("info");
  const isImporting = ref(false);

  const projectOptions = computed(() => {
    if (!initialProject || defaultProjects.some((p) => p.value === initialProject)) {
      return defaultProjects;
    }
    return [{ label: initialProject, value: initialProject }, ...defaultProjects];
  });

  const selectedProject = computed(() =>
    projectOptions.value.find((p) => p.value === projectCode.value),
  );

  watch(
    () => initialProject,
    (val) => {
      projectCode.value = val;
    },
  );

  function selectFile(file: File | null) {
    selectedFile.value = file;
    rows.value = [];
    message.value = file ? `${file.name} selected. Click Import to preview issues.` : emptyImportMessage;
    messageMode.value = "info";
  }

  async function importCsv() {
    if (!projectCode.value) {
      message.value = "Please select a project before importing issues.";
      messageMode.value = "error";
      return;
    }

    if (!selectedFile.value) {
      message.value = "Please select an issue CSV file before importing.";
      messageMode.value = "error";
      return;
    }

    isImporting.value = true;
    message.value = "Importing issue CSV...";
    messageMode.value = "info";

    try {
      const content = await selectedFile.value.text();
      const result = parseIssueCsv(content);
      rows.value = result;
      message.value = `Imported ${result.length.toLocaleString("en-US")} issues for ${selectedProject.value?.label ?? projectCode.value}.`;
      messageMode.value = "info";
    } catch (error) {
      rows.value = [];
      message.value = error instanceof Error ? error.message : "Could not import the selected issue CSV file.";
      messageMode.value = "error";
    } finally {
      isImporting.value = false;
    }
  }

  return {
    importCsv,
    isImporting,
    message,
    messageMode,
    projectCode,
    projectOptions,
    rows,
    selectFile,
    selectedFile,
    selectedProject,
  };
}

function parseIssueCsv(content: string): IssueCsvRow[] {
  const records = parseCsvRecords(content);
  const [headers, ...dataRows] = records.filter((record) => record.some((cell) => cell.trim()));

  if (!headers) return [];

  const normalizedHeaders = headers.map((h) => h.trim());
  const hasExpectedHeaders = issueCsvHeaders.every((h, i) => normalizedHeaders[i] === h);
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
      if (char === "\r" && nextChar === "\n") index += 1;
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

export function priorityTone(priority: string) {
  if (priority === "High") return "bg-orange-100 text-orange-800";
  if (priority === "Normal") return "bg-sky-100 text-sky-800";
  if (priority === "Low") return "bg-slate-100 text-slate-700";
  return "bg-red-100 text-red-800";
}
