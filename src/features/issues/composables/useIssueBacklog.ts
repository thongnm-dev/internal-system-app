import { computed, ref } from "vue";

type IssueStatus = "All" | "Open" | "In Progress" | "Review" | "Resolved" | "Closed";

export type BacklogSearchCriteria = {
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

export type IssueBacklogItem = {
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

export const statusOptions: IssueStatus[] = ["All", "Open", "In Progress", "Review", "Resolved", "Closed"];
export const projects = ["Billing Portal", "Internal Extension", "Mobile Gateway", "Reporting Hub"];
export const issueTypes = ["Bug", "Task", "Story", "Improvement"];
export const categories = ["Backend", "Frontend", "Database", "Integration", "Operation"];
export const assignees = ["An Nguyen", "Bao Tran", "Chi Le", "Dung Pham", "Linh Ho"];
export const bugClasses = ["Functional", "UI", "Performance", "Security", "Data"];

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

export const uniqueCreateUsers = Array.from(new Set(backlogItems.map((i) => i.createUser))).sort();

function normalize(value: string) {
  return value.trim().toLocaleLowerCase();
}

function matchesCriteria(item: IssueBacklogItem, criteria: BacklogSearchCriteria) {
  const keyword = normalize(criteria.keyword);
  const matchesKeyword =
    !keyword ||
    normalize([item.issueKey, item.subject, item.assignee, item.createUser, item.project, item.category].join(" ")).includes(keyword);

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

export function statusTone(status: string) {
  if (status === "Open") return "bg-blue-100 text-blue-800";
  if (status === "In Progress") return "bg-amber-100 text-amber-800";
  if (status === "Review") return "bg-indigo-100 text-indigo-800";
  if (status === "Resolved") return "bg-emerald-100 text-emerald-800";
  return "bg-slate-100 text-slate-700";
}

export function priorityTone(priority: string) {
  if (priority === "Critical") return "bg-red-100 text-red-800";
  if (priority === "High") return "bg-orange-100 text-orange-800";
  if (priority === "Medium") return "bg-sky-100 text-sky-800";
  return "bg-slate-100 text-slate-700";
}

export function useIssueBacklog() {
  const criteria = ref<BacklogSearchCriteria>({ ...initialCriteria });
  const appliedCriteria = ref<BacklogSearchCriteria>({ ...initialCriteria });

  const filteredItems = computed(() => backlogItems.filter((item) => matchesCriteria(item, appliedCriteria.value)));

  const canOpenImport = computed(() => Boolean(criteria.value.project));

  function setField<Field extends keyof BacklogSearchCriteria>(field: Field, value: BacklogSearchCriteria[Field]) {
    criteria.value = { ...criteria.value, [field]: value };
  }

  function search() {
    appliedCriteria.value = { ...criteria.value };
  }

  function reset() {
    criteria.value = { ...initialCriteria };
    appliedCriteria.value = { ...initialCriteria };
  }

  return { criteria, filteredItems, canOpenImport, setField, search, reset };
}
