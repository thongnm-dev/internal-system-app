import { computed, ref, watch } from "vue";
import { listProjects } from "@/tauri/commands/project";
import {
  backlogListStatuses,
  backlogListCategories,
  backlogListIssueTypes,
  type BacklogStatus,
  type BacklogCategory,
  type BacklogIssueType,
} from "@/tauri/commands/backlog";
import type { ProjectSummaryResult } from "@/_/types/project";
import { canUseTauriRuntime } from "@/tauri/commands/_base";

export type BacklogSearchCriteria = {
  project: string;
  status: string[];
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
  status: string;
  hours: number;
  priority: string;
  createDate: string;
  createUser: string;
  project: string;
  category: string;
  bugClass: string;
};

const initialCriteria: BacklogSearchCriteria = {
  project: "",
  status: [],
  issueType: "",
  category: "",
  assignee: "",
  keyword: "",
  createDateFrom: "",
  createDateTo: "",
  createUser: "",
  bugClass: "",
};

export const projects = ref<ProjectSummaryResult[]>([]);
export const statusOptions = ref<string[]>([]);
export const issueTypes = ref<string[]>([]);
export const categories = ref<string[]>([]);
export const assignees = ref<string[]>([]);
export const bugClasses = ref<string[]>([]);
export const uniqueCreateUsers = ref<string[]>([]);

const lookupLoading = ref(false);
const lookupError = ref("");

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
    (criteria.status.length === 0 || criteria.status.includes(item.status)) &&
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
  const s = status.toLocaleLowerCase();
  if (s.includes("open")) return "bg-blue-100 text-blue-800";
  if (s.includes("progress") || s.includes("処理中")) return "bg-amber-100 text-amber-800";
  if (s.includes("review") || s.includes("処理済み")) return "bg-indigo-100 text-indigo-800";
  if (s.includes("resolved") || s.includes("完了")) return "bg-emerald-100 text-emerald-800";
  if (s.includes("closed") || s.includes("close")) return "bg-slate-100 text-slate-700";
  return "bg-slate-100 text-slate-700";
}

export function priorityTone(priority: string) {
  const p = priority.toLocaleLowerCase();
  if (p.includes("critical") || p === "高") return "bg-red-100 text-red-800";
  if (p.includes("high")) return "bg-orange-100 text-orange-800";
  if (p.includes("medium") || p === "中") return "bg-sky-100 text-sky-800";
  return "bg-slate-100 text-slate-700";
}

async function loadProjects() {
  if (!canUseTauriRuntime()) return;
  try {
    const list = await listProjects();
    projects.value = list.filter((p) => p.is_active);
  } catch {
    projects.value = [];
  }
}

function resolveBacklogKey(projectCode: string): string | null {
  if (!projectCode) return null;
  const proj = projects.value.find((p) => p.code === projectCode);
  if (!proj) return null;
  // We need the backlog_key from ProjectDetail, but ProjectSummary doesn't have it.
  // The backlog_key is part of the full detail. We'll load it separately.
  return null;
}

async function loadProjectLookups(backlogKey: string) {
  if (!backlogKey) {
    statusOptions.value = ["All"];
    issueTypes.value = [];
    categories.value = [];
    lookupError.value = "";
    return;
  }

  lookupLoading.value = true;
  lookupError.value = "";

  try {
    const [statuses, types, cats] = await Promise.all([
      backlogListStatuses(backlogKey),
      backlogListIssueTypes(backlogKey),
      backlogListCategories(backlogKey),
    ]);

    statusOptions.value = statuses.map((s: BacklogStatus) => s.name);
    issueTypes.value = types.map((t: BacklogIssueType) => t.name);
    categories.value = cats.map((c: BacklogCategory) => c.name);
  } catch (e) {
    lookupError.value = String(e);
    statusOptions.value = ["All"];
    issueTypes.value = [];
    categories.value = [];
  } finally {
    lookupLoading.value = false;
  }
}

export function useIssueBacklog(initialProject = "") {
  const startingCriteria: BacklogSearchCriteria = { ...initialCriteria, project: initialProject };
  const criteria = ref<BacklogSearchCriteria>({ ...startingCriteria });
  const appliedCriteria = ref<BacklogSearchCriteria>({ ...startingCriteria });
  const backlogItems = ref<IssueBacklogItem[]>([]);

  const filteredItems = computed(() => backlogItems.value.filter((item) => matchesCriteria(item, appliedCriteria.value)));

  const canOpenImport = computed(() => Boolean(criteria.value.project));

  const projectBacklogKeys = ref<Map<string, string>>(new Map());

  async function loadProjectBacklogKey(projectCode: string): Promise<string> {
    if (projectBacklogKeys.value.has(projectCode)) {
      return projectBacklogKeys.value.get(projectCode)!;
    }

    const proj = projects.value.find((p) => p.code === projectCode);
    if (!proj) return "";

    try {
      const { getProjectDetail } = await import("@/tauri/commands/project");
      const detail = await getProjectDetail(proj.id);
      const key = detail.backlog_key || "";
      projectBacklogKeys.value.set(projectCode, key);
      return key;
    } catch {
      return "";
    }
  }

  function setField<Field extends keyof BacklogSearchCriteria>(field: Field, value: BacklogSearchCriteria[Field]) {
    criteria.value = { ...criteria.value, [field]: value };
  }

  watch(
    () => criteria.value.project,
    async (projectCode) => {
      criteria.value = { ...criteria.value, status: [], issueType: "", category: "" };

      if (!projectCode) {
        await loadProjectLookups("");
        return;
      }

      const backlogKey = await loadProjectBacklogKey(projectCode);
      if (backlogKey) {
        await loadProjectLookups(backlogKey);
      } else {
        await loadProjectLookups("");
      }
    },
  );

  function search() {
    appliedCriteria.value = { ...criteria.value };
  }

  function reset() {
    criteria.value = { ...initialCriteria };
    appliedCriteria.value = { ...initialCriteria };
    loadProjectLookups("");
  }

  loadProjects();

  if (initialProject) {
    loadProjectBacklogKey(initialProject).then((key) => {
      if (key) loadProjectLookups(key);
    });
  }

  return { criteria, filteredItems, canOpenImport, setField, search, reset, lookupLoading, lookupError };
}
