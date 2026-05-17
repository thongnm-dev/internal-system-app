import { Calendar as IconCalendar, RotateCcw, Search } from "lucide-react";
import { Button } from "primereact/button";
import { Calendar } from "primereact/calendar";
import { Column } from "primereact/column";
import { DataTable } from "primereact/datatable";
import { Dropdown } from "primereact/dropdown";

import { Fieldset } from "primereact/fieldset";
import { InputText } from "primereact/inputtext";
import { SelectButton } from "primereact/selectbutton";
import { useMemo, useState } from "react";

type BacklogSearchCriteria = {
  project: string;
  status: IssueStatus;
  issueType: string;
  category: string;
  assignee: string;
  keyword: string;
  createDateFrom: Date | null;
  createDateTo: Date | null;
  createUser: string;
  bugClass: string;
};

type IssueStatus = "All" | "Open" | "In Progress" | "Review" | "Resolved" | "Closed";

type IssueBacklogItem = {
  id: number;
  issueType: string;
  issueKey: string;
  subject: string;
  assignee: string;
  status: Exclude<IssueStatus, "All">;
  hours: number;
  priority: "Critical" | "High" | "Medium" | "Low";
  createDate: string;
  createUser: string;
  project: string;
  category: string;
  bugClass: string;
};

const statusOptions: IssueStatus[] = ["All", "Open", "In Progress", "Review", "Resolved", "Closed"];
const projects = ["Billing Portal", "Internal Extension", "Mobile Gateway", "Reporting Hub"];
const issueTypes = ["Bug", "Task", "Story", "Improvement"];
const categories = ["Backend", "Frontend", "Database", "Integration", "Operation"];
const assignees = ["An Nguyen", "Bao Tran", "Chi Le", "Dung Pham", "Linh Ho"];
const bugClasses = ["Functional", "UI", "Performance", "Security", "Data"];

const initialCriteria: BacklogSearchCriteria = {
  project: "",
  status: "All",
  issueType: "",
  category: "",
  assignee: "",
  keyword: "",
  createDateFrom: null,
  createDateTo: null,
  createUser: "",
  bugClass: "",
};

const backlogItems: IssueBacklogItem[] = [
  {
    id: 1,
    issueType: "Bug",
    issueKey: "INT-1042",
    subject: "Import CSV preview shows duplicated monthly totals",
    assignee: "An Nguyen",
    status: "Open",
    hours: 6.5,
    priority: "High",
    createDate: "2026-05-14",
    createUser: "Minh Hoang",
    project: "Internal Extension",
    category: "Backend",
    bugClass: "Data",
  },
  {
    id: 2,
    issueType: "Task",
    issueKey: "BIL-883",
    subject: "Add payment reconciliation audit export",
    assignee: "Bao Tran",
    status: "In Progress",
    hours: 12,
    priority: "Medium",
    createDate: "2026-05-12",
    createUser: "Hana Ito",
    project: "Billing Portal",
    category: "Backend",
    bugClass: "",
  },
  {
    id: 3,
    issueType: "Bug",
    issueKey: "MOB-512",
    subject: "Token refresh fails after switching network",
    assignee: "Chi Le",
    status: "Review",
    hours: 4,
    priority: "Critical",
    createDate: "2026-05-10",
    createUser: "Quang Pham",
    project: "Mobile Gateway",
    category: "Integration",
    bugClass: "Security",
  },
  {
    id: 4,
    issueType: "Story",
    issueKey: "REP-219",
    subject: "Create team workload dashboard summary",
    assignee: "Dung Pham",
    status: "Resolved",
    hours: 18,
    priority: "High",
    createDate: "2026-05-09",
    createUser: "Linh Ho",
    project: "Reporting Hub",
    category: "Frontend",
    bugClass: "",
  },
  {
    id: 5,
    issueType: "Improvement",
    issueKey: "INT-1019",
    subject: "Tighten keyboard flow in daily report input grid",
    assignee: "Linh Ho",
    status: "Closed",
    hours: 3.5,
    priority: "Low",
    createDate: "2026-05-04",
    createUser: "An Nguyen",
    project: "Internal Extension",
    category: "Frontend",
    bugClass: "UI",
  },
];

export function IssueBacklogPage() {
  const [criteria, setCriteria] = useState(initialCriteria);
  const [appliedCriteria, setAppliedCriteria] = useState(initialCriteria);

  const filteredItems = useMemo(
    () => backlogItems.filter((item) => matchesCriteria(item, appliedCriteria)),
    [appliedCriteria],
  );

  const setField = <Field extends keyof BacklogSearchCriteria>(field: Field, value: BacklogSearchCriteria[Field]) => {
    setCriteria((current) => ({ ...current, [field]: value }));
  };

  const reset = () => {
    setCriteria(initialCriteria);
    setAppliedCriteria(initialCriteria);
  };

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-1 overflow-x-hidden overflow-y-auto">
      <Fieldset
        className="rounded-lg border border-slate-200 bg-white p-4 shadow-md fieldset-nested"
        legend="Search"
        toggleable
      >
        <div className="grid gap-3">
          <div className="grid items-end gap-3 lg:grid-cols-2">
            <SelectField
              label="Project"
              value={criteria.project}
              options={projects}
              placeholder="All projects"
              onChange={(value) => setField("project", value)}
            />

            <label className="block min-w-0">
              <span className="text-xs font-bold text-slate-500">Status</span>
              <SelectButton
                className="mt-1 grid h-10 min-w-0 grid-cols-6 gap-1 rounded-md border border-slate-300 bg-white p-1 text-xs leading-none [&_.p-button]:min-w-0 [&_.p-button]:justify-center [&_.p-button]:px-1.5 [&_.p-button]:py-0 [&_.p-button]:text-xs [&_.p-button-label]:truncate [&_.p-button-label]:text-xs [&_.p-button-label]:font-normal"
                value={criteria.status}
                options={statusOptions}
                allowEmpty={false}
                onChange={(event) => {
                  if (event.value) {
                    setField("status", event.value);
                  }
                }}
              />
            </label>
          </div>

          <div className="grid gap-3 lg:grid-cols-2">
            <SelectField
              label="Issue Type"
              value={criteria.issueType}
              options={issueTypes}
              placeholder="All types"
              onChange={(value) => setField("issueType", value)}
            />
            <div className="grid gap-3 md:grid-cols-2">
              <SelectField
                label="Category"
                value={criteria.category}
                options={categories}
                placeholder="All categories"
                onChange={(value) => setField("category", value)}
              />
              <SelectField
                label="Assignee"
                value={criteria.assignee}
                options={assignees}
                placeholder="All assignees"
                onChange={(value) => setField("assignee", value)}
              />
            </div>
          </div>

          <div className="grid gap-3 lg:grid-cols-2">
            <TextField
              label="Keyword"
              value={criteria.keyword}
              placeholder="Issue key or subject"
              type="search"
              onChange={(value) => setField("keyword", value)}
              onEnter={() => setAppliedCriteria(criteria)}
            />
            <div />
          </div>

          <Fieldset
            className="rounded-md border border-slate-300 bg-white p-3 fieldset-nested"
            legend="Advanced"
            toggleable

          >
            <div className="grid gap-3 lg:grid-cols-2">
              <div className="grid gap-3 md:grid-cols-2">
                <label className="block min-w-0">
                  <span className="text-xs font-bold text-slate-500">Create date from</span>
                  <Calendar
                    ariaLabel="Create date from"
                    className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white text-sm text-slate-900 outline-none focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100"
                    icon={<IconCalendar className="text-slate-400" />}
                    placeholder="mm/dd/yyyy"
                    showIcon
                    value={criteria.createDateFrom}
                    onChange={(event) => setField("createDateFrom", event.value instanceof Date ? event.value : null)}
                  />
                </label>
                <label className="block min-w-0">
                  <span className="text-xs font-bold text-slate-500">Create date to</span>
                  <Calendar
                    ariaLabel="Create date to"
                    className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white text-sm text-slate-900 outline-none focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100"
                    icon={<IconCalendar className="text-slate-400" />}
                    placeholder="mm/dd/yyyy"
                    showIcon
                    value={criteria.createDateTo}
                    onChange={(event) => setField("createDateTo", event.value instanceof Date ? event.value : null)}
                  />
                </label>
              </div>
              <div className="grid gap-3 md:grid-cols-2">
                <SelectField
                  label="Create user"
                  value={criteria.createUser}
                  options={uniqueCreateUsers}
                  placeholder="All users"
                  onChange={(value) => setField("createUser", value)}
                />
                <SelectField
                  label="Bug class"
                  value={criteria.bugClass}
                  options={bugClasses}
                  placeholder="All bug classes"
                  onChange={(value) => setField("bugClass", value)}
                />
              </div>
            </div>
          </Fieldset>

          <div className="flex items-center justify-end gap-2">
            <Button
              className="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90"
              type="button"
              title="Search"
              onClick={() => setAppliedCriteria(criteria)}
            >
              <Search className="h-4 w-4" />
              Search
            </Button>
            <Button
              className="flex h-10 items-center gap-2 rounded-md border border-slate-300 bg-white px-4 text-sm font-bold text-slate-600 hover:bg-slate-50"
              type="button"
              title="Reset search conditions"
              onClick={reset}
            >
              <RotateCcw className="h-4 w-4" />
              Reset
            </Button>
          </div>
        </div>
      </Fieldset>

      <section className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-stone-200 bg-panel shadow-sm">
        <div className="flex items-center justify-between gap-4 border-b border-stone-200 px-4 py-3">
          <h3 className="font-bold">Issue backlog list</h3>
          <span className="text-xs text-slate-500">{filteredItems.length.toLocaleString("en-US")} issues</span>
        </div>

        <DataTable
          className="app-data-table min-h-0"
          emptyMessage="No issues match the search conditions."
          scrollable
          scrollHeight="flex"
          tableStyle={{ minWidth: "1180px" }}
          value={filteredItems}
        >
          <Column field="issueType" header="Issue Type" bodyClassName="whitespace-nowrap" />
          <Column field="issueKey" header="Issue Key" bodyClassName="whitespace-nowrap font-bold text-ink" />
          <Column field="subject" header="Subject" bodyClassName="max-w-[360px] truncate" />
          <Column field="assignee" header="Assignee" bodyClassName="whitespace-nowrap" />
          <Column field="status" header="Status" body={statusBody} bodyClassName="whitespace-nowrap" />
          <Column
            field="hours"
            header="Hours"
            body={(item: IssueBacklogItem) => item.hours.toFixed(item.hours % 1 === 0 ? 0 : 1)}
            bodyClassName="num"
            headerClassName="num"
          />
          <Column field="priority" header="Priority" body={priorityBody} bodyClassName="whitespace-nowrap" />
          <Column field="createDate" header="Create Date" bodyClassName="whitespace-nowrap" />
          <Column field="createUser" header="Create User" bodyClassName="whitespace-nowrap" />
        </DataTable>
      </section>
    </section>
  );
}

const uniqueCreateUsers = Array.from(new Set(backlogItems.map((item) => item.createUser))).sort();

function SelectField({
  label,
  onChange,
  options,
  placeholder,
  value,
}: {
  label: string;
  onChange: (value: string) => void;
  options: string[];
  placeholder: string;
  value: string;
}) {
  return (
    <label className="block min-w-0">
      <span className="text-xs font-bold text-slate-500">{label}</span>
      <Dropdown
        className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-xs text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100 [&_.p-dropdown-label]:text-xs"
        options={options}
        placeholder={placeholder}
        value={value}
        onChange={(event) => onChange(event.value ?? "")}
      />
    </label>
  );
}

function TextField({
  label,
  onChange,
  onEnter,
  placeholder,
  type,
  value,
}: {
  label: string;
  onChange: (value: string) => void;
  onEnter?: () => void;
  placeholder: string;
  type: "search" | "text";
  value: string;
}) {
  return (
    <label className="block min-w-0">
      <span className="text-xs font-bold text-slate-500">{label}</span>
      <InputText
        className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
        placeholder={placeholder}
        type={type}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            onEnter?.();
          }
        }}
      />
    </label>
  );
}

function statusBody(item: IssueBacklogItem) {
  return (
    <span className={["rounded px-2 py-1 text-xs font-bold", statusTone(item.status)].join(" ")}>
      {item.status}
    </span>
  );
}

function priorityBody(item: IssueBacklogItem) {
  return (
    <span className={["rounded px-2 py-1 text-xs font-bold", priorityTone(item.priority)].join(" ")}>
      {item.priority}
    </span>
  );
}

function matchesCriteria(item: IssueBacklogItem, criteria: BacklogSearchCriteria) {
  const keyword = normalize(criteria.keyword);
  const createDateFrom = formatDateKey(criteria.createDateFrom);
  const createDateTo = formatDateKey(criteria.createDateTo);
  const matchesKeyword =
    !keyword ||
    normalize([item.issueKey, item.subject, item.assignee, item.createUser, item.project, item.category].join(" ")).includes(
      keyword,
    );

  return (
    (!criteria.project || item.project === criteria.project) &&
    (criteria.status === "All" || item.status === criteria.status) &&
    (!criteria.issueType || item.issueType === criteria.issueType) &&
    (!criteria.category || item.category === criteria.category) &&
    (!criteria.assignee || item.assignee === criteria.assignee) &&
    matchesKeyword &&
    (!createDateFrom || item.createDate >= createDateFrom) &&
    (!createDateTo || item.createDate <= createDateTo) &&
    (!criteria.createUser || item.createUser === criteria.createUser) &&
    (!criteria.bugClass || item.bugClass === criteria.bugClass)
  );
}

function formatDateKey(value: Date | null) {
  if (!value) {
    return "";
  }

  const year = value.getFullYear();
  const month = String(value.getMonth() + 1).padStart(2, "0");
  const day = String(value.getDate()).padStart(2, "0");

  return `${year}-${month}-${day}`;
}

function statusTone(status: IssueBacklogItem["status"]) {
  if (status === "Open") {
    return "bg-blue-100 text-blue-800";
  }

  if (status === "In Progress") {
    return "bg-amber-100 text-amber-800";
  }

  if (status === "Review") {
    return "bg-indigo-100 text-indigo-800";
  }

  if (status === "Resolved") {
    return "bg-emerald-100 text-emerald-800";
  }

  return "bg-slate-100 text-slate-700";
}

function priorityTone(priority: IssueBacklogItem["priority"]) {
  if (priority === "Critical") {
    return "bg-red-100 text-red-800";
  }

  if (priority === "High") {
    return "bg-orange-100 text-orange-800";
  }

  if (priority === "Medium") {
    return "bg-sky-100 text-sky-800";
  }

  return "bg-slate-100 text-slate-700";
}

function normalize(value: string) {
  return value.trim().toLocaleLowerCase();
}
