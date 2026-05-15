import { Calendar, RotateCcw, Search } from "lucide-react";
import { useMemo, useState } from "react";

type BacklogSearchCriteria = {
  project: string;
  status: IssueStatus;
  issueType: string;
  category: string;
  assignee: string;
  keyword: string;
  createDateFrom: string;
  createDateTo: string;
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
  createDateFrom: "",
  createDateTo: "",
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
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <fieldset className="rounded-lg border border-stone-200 bg-panel p-4 shadow-sm">
        <legend className="px-2 text-sm font-bold text-slate-600">Search conditions</legend>

        <div className="grid gap-3">
          <div className="grid gap-3 lg:grid-cols-2">
            <SelectField
              label="Project"
              value={criteria.project}
              options={projects}
              placeholder="All projects"
              onChange={(value) => setField("project", value)}
            />
            <div />
          </div>

          <div className="grid gap-3 lg:grid-cols-2">
            <StatusSwitch value={criteria.status} onChange={(value) => setField("status", value)} />
            <div />
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

          <fieldset className="rounded-md border border-slate-200 bg-slate-50/70 p-3">
            <legend className="px-2 text-xs font-bold text-slate-500">Advanced search</legend>
            <div className="grid gap-3 lg:grid-cols-2">
              <div className="grid gap-3 md:grid-cols-2">
                <DateField
                  label="Create date from"
                  value={criteria.createDateFrom}
                  onChange={(value) => setField("createDateFrom", value)}
                />
                <DateField
                  label="Create date to"
                  value={criteria.createDateTo}
                  onChange={(value) => setField("createDateTo", value)}
                />
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
          </fieldset>

          <div className="flex items-center justify-end gap-2">
            <button
              className="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90"
              type="button"
              title="Search"
              onClick={() => setAppliedCriteria(criteria)}
            >
              <Search className="h-4 w-4" />
              Search
            </button>
            <button
              className="flex h-10 items-center gap-2 rounded-md border border-slate-300 bg-white px-4 text-sm font-bold text-slate-600 hover:bg-slate-50"
              type="button"
              title="Reset search conditions"
              onClick={reset}
            >
              <RotateCcw className="h-4 w-4" />
              Reset
            </button>
          </div>
        </div>
      </fieldset>

      <section className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-stone-200 bg-panel shadow-sm">
        <div className="flex items-center justify-between gap-4 border-b border-stone-200 px-4 py-3">
          <h3 className="font-bold">Issue backlog list</h3>
          <span className="text-xs text-slate-500">{filteredItems.length.toLocaleString("en-US")} issues</span>
        </div>

        <div className="min-h-0 overflow-auto">
          <table className="w-full min-w-[1180px] border-collapse">
            <thead>
              <tr>
                <th className="table-head">Issue Type</th>
                <th className="table-head">Issue Key</th>
                <th className="table-head">Subject</th>
                <th className="table-head">Assignee</th>
                <th className="table-head">Status</th>
                <th className="table-head num">Hours</th>
                <th className="table-head">Priority</th>
                <th className="table-head">Create Date</th>
                <th className="table-head">Create User</th>
              </tr>
            </thead>
            <tbody>
              {filteredItems.length === 0 ? (
                <tr>
                  <td className="table-cell h-40 text-center text-slate-500" colSpan={9}>
                    No issues match the search conditions.
                  </td>
                </tr>
              ) : (
                filteredItems.map((item) => <IssueRow key={item.id} item={item} />)
              )}
            </tbody>
          </table>
        </div>
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
      <select
        className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
        value={value}
        onChange={(event) => onChange(event.target.value)}
      >
        <option value="">{placeholder}</option>
        {options.map((option) => (
          <option key={option} value={option}>
            {option}
          </option>
        ))}
      </select>
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
      <input
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

function DateField({ label, onChange, value }: { label: string; onChange: (value: string) => void; value: string }) {
  return (
    <label className="block min-w-0">
      <span className="text-xs font-bold text-slate-500">{label}</span>
      <span className="relative mt-1 block">
        <input
          className="h-10 w-full rounded-md border border-slate-300 bg-white px-3 pr-10 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
          type="date"
          value={value}
          onChange={(event) => onChange(event.target.value)}
        />
        <Calendar className="pointer-events-none absolute right-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
      </span>
    </label>
  );
}

function StatusSwitch({ onChange, value }: { onChange: (value: IssueStatus) => void; value: IssueStatus }) {
  return (
    <div className="min-w-0">
      <span className="text-xs font-bold text-slate-500">Status</span>
      <div className="mt-1 grid grid-cols-3 gap-1 rounded-md border border-slate-300 bg-white p-1 md:grid-cols-6">
        {statusOptions.map((status) => {
          const isActive = status === value;
          return (
            <button
              key={status}
              className={[
                "h-8 rounded px-2 text-xs font-bold transition",
                isActive ? "bg-brand text-white shadow-sm" : "text-slate-600 hover:bg-slate-100",
              ].join(" ")}
              type="button"
              onClick={() => onChange(status)}
            >
              {status}
            </button>
          );
        })}
      </div>
    </div>
  );
}

function IssueRow({ item }: { item: IssueBacklogItem }) {
  return (
    <tr className="hover:bg-slate-50">
      <td className="table-cell whitespace-nowrap">{item.issueType}</td>
      <td className="table-cell whitespace-nowrap font-bold text-ink">{item.issueKey}</td>
      <td className="table-cell max-w-[360px] truncate">{item.subject}</td>
      <td className="table-cell whitespace-nowrap">{item.assignee}</td>
      <td className="table-cell whitespace-nowrap">
        <span className={["rounded px-2 py-1 text-xs font-bold", statusTone(item.status)].join(" ")}>
          {item.status}
        </span>
      </td>
      <td className="table-cell num">{item.hours.toFixed(item.hours % 1 === 0 ? 0 : 1)}</td>
      <td className="table-cell whitespace-nowrap">
        <span className={["rounded px-2 py-1 text-xs font-bold", priorityTone(item.priority)].join(" ")}>
          {item.priority}
        </span>
      </td>
      <td className="table-cell whitespace-nowrap">{item.createDate}</td>
      <td className="table-cell whitespace-nowrap">{item.createUser}</td>
    </tr>
  );
}

function matchesCriteria(item: IssueBacklogItem, criteria: BacklogSearchCriteria) {
  const keyword = normalize(criteria.keyword);
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
    (!criteria.createDateFrom || item.createDate >= criteria.createDateFrom) &&
    (!criteria.createDateTo || item.createDate <= criteria.createDateTo) &&
    (!criteria.createUser || item.createUser === criteria.createUser) &&
    (!criteria.bugClass || item.bugClass === criteria.bugClass)
  );
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
