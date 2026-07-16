import { computed, ref, watch } from "vue";
import { listProjects } from "@/tauri/commands/project";
import {
  backlogListStatuses,
  backlogListCategories,
  backlogListIssueTypes,
  backlogListProjectUsers,
  backlogListPriorities,
  backlogListIssues,
  backlogGetProjectLookup,
  type BacklogStatus,
  type BacklogCategory,
  type BacklogIssueType,
  type BacklogUser,
  type BacklogPriority,
  type BacklogIssue,
} from "@/tauri/commands/backlog";
import type { ProjectSummaryResult } from "@/_/types/project";
import { canUseTauriRuntime } from "@/tauri/commands/_base";

export type BacklogSearchCriteria = {
  project: string;
  status: string[];
  notClosed: boolean;
  issueType: string;
  category: string;
  assignee: string;
  keyword: string;
  createDateFrom: string;
  createDateTo: string;
  createUser: string;
  priorityFilter: string;
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
};

const initialCriteria: BacklogSearchCriteria = {
  project: "",
  status: [],
  notClosed: false,
  issueType: "",
  category: "",
  assignee: "",
  keyword: "",
  createDateFrom: "",
  createDateTo: "",
  createUser: "",
  priorityFilter: "",
};

export const projects = ref<ProjectSummaryResult[]>([]);
export const statusOptions = ref<string[]>([]);
export const issueTypes = ref<string[]>([]);
export const categories = ref<string[]>([]);
export const assignees = ref<string[]>([]);
export const priorityOptions = ref<string[]>([]);
export const uniqueCreateUsers = ref<string[]>([]);

const lookupLoading = ref(false);
const lookupError = ref("");

const statusLookup = ref<BacklogStatus[]>([]);
const issueTypeLookup = ref<BacklogIssueType[]>([]);
const categoryLookup = ref<BacklogCategory[]>([]);
const userLookup = ref<BacklogUser[]>([]);
const priorityLookup = ref<BacklogPriority[]>([]);

function formatDate(dateStr: string | null | undefined): string {
  if (!dateStr) return "";
  return dateStr.substring(0, 10);
}

function mapIssue(issue: BacklogIssue): IssueBacklogItem {
  return {
    id: issue.id,
    issueType: issue.issueType?.name ?? "",
    issueKey: issue.issueKey,
    subject: issue.summary,
    assignee: issue.assignee?.name ?? "",
    status: issue.status?.name ?? "",
    hours: issue.actualHours ?? issue.estimatedHours ?? 0,
    priority: issue.priority?.name ?? "",
    createDate: formatDate(issue.created),
    createUser: issue.createdUser?.name ?? "",
    project: issue.issueKey.split("-")[0] ?? "",
    category: issue.category?.[0]?.name ?? "",
  };
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

async function loadProjectLookups(backlogKey: string) {
  if (!backlogKey) {
    statusOptions.value = [];
    issueTypes.value = [];
    categories.value = [];
    assignees.value = [];
    priorityOptions.value = [];
    statusLookup.value = [];
    issueTypeLookup.value = [];
    categoryLookup.value = [];
    userLookup.value = [];
    priorityLookup.value = [];
    lookupError.value = "";
    return;
  }

  lookupLoading.value = true;
  lookupError.value = "";

  try {
    const [statuses, types, cats, users, prios] = await Promise.all([
      backlogListStatuses(backlogKey),
      backlogListIssueTypes(backlogKey),
      backlogListCategories(backlogKey),
      backlogListProjectUsers(backlogKey),
      backlogListPriorities(),
    ]);

    statusLookup.value = statuses;
    issueTypeLookup.value = types;
    categoryLookup.value = cats;
    userLookup.value = users;
    priorityLookup.value = prios;

    statusOptions.value = statuses.map((s: BacklogStatus) => s.name);
    issueTypes.value = types.map((t: BacklogIssueType) => t.name);
    categories.value = cats.map((c: BacklogCategory) => c.name);
    assignees.value = users.map((u: BacklogUser) => u.name);
    priorityOptions.value = prios.map((p: BacklogPriority) => p.name);
  } catch (e) {
    lookupError.value = String(e);
    statusOptions.value = [];
    issueTypes.value = [];
    categories.value = [];
    assignees.value = [];
    priorityOptions.value = [];
    statusLookup.value = [];
    issueTypeLookup.value = [];
    categoryLookup.value = [];
    userLookup.value = [];
    priorityLookup.value = [];
  } finally {
    lookupLoading.value = false;
  }
}

export function useIssueBacklog(initialProject = "") {
  const startingCriteria: BacklogSearchCriteria = { ...initialCriteria, project: initialProject };
  const criteria = ref<BacklogSearchCriteria>({ ...startingCriteria });
  const appliedCriteria = ref<BacklogSearchCriteria>({ ...startingCriteria });
  const backlogItems = ref<IssueBacklogItem[]>([]);
  const searching = ref(false);
  const searchError = ref("");
  const totalCount = ref(0);
  const first = ref(0);
  const pageSize = ref(20);

  const filteredItems = computed(() => backlogItems.value);

  const canOpenImport = computed(() => Boolean(criteria.value.project));

  const projectBacklogKeys = ref<Map<string, string>>(new Map());
  const projectIds = ref<Map<string, string>>(new Map());

  async function resolveProjectInfo(projectCode: string): Promise<{ backlogKey: string; projectId: string }> {
    if (projectBacklogKeys.value.has(projectCode)) {
      return {
        backlogKey: projectBacklogKeys.value.get(projectCode)!,
        projectId: projectIds.value.get(projectCode) || "",
      };
    }

    const proj = projects.value.find((p) => p.code === projectCode);
    if (!proj) return { backlogKey: "", projectId: "" };

    try {
      const { getProjectDetail } = await import("@/tauri/commands/project");
      const detail = await getProjectDetail(proj.id);
      const key = detail.backlog_key || "";
      projectBacklogKeys.value.set(projectCode, key);

      if (key) {
        const lookup = await backlogGetProjectLookup(key);
        projectIds.value.set(projectCode, lookup.projectId);
        return { backlogKey: key, projectId: lookup.projectId };
      }

      return { backlogKey: key, projectId: "" };
    } catch {
      return { backlogKey: "", projectId: "" };
    }
  }

  function setField<Field extends keyof BacklogSearchCriteria>(field: Field, value: BacklogSearchCriteria[Field]) {
    criteria.value = { ...criteria.value, [field]: value };
  }

  watch(
    () => criteria.value.project,
    async (projectCode) => {
      criteria.value = { ...criteria.value, status: [], notClosed: false, issueType: "", category: "", assignee: "", priorityFilter: "" };

      if (!projectCode) {
        await loadProjectLookups("");
        return;
      }

      const { backlogKey } = await resolveProjectInfo(projectCode);
      if (backlogKey) {
        await loadProjectLookups(backlogKey);
      } else {
        await loadProjectLookups("");
      }
    },
  );

  async function fetchPage(offset: number, count: number) {
    const c = appliedCriteria.value;

    const { projectId } = await resolveProjectInfo(c.project);
    if (!projectId) {
      searchError.value = "Could not resolve project ID for Backlog API.";
      backlogItems.value = [];
      totalCount.value = 0;
      return;
    }

    searching.value = true;
    searchError.value = "";

    try {
      let statusIds: number[] | undefined;
      if (c.notClosed) {
        statusIds = statusLookup.value
          .filter((s) => {
            const n = s.name.toLowerCase();
            return !(n.includes("closed") || n.includes("close") || n === "完了");
          })
          .map((s) => s.id);
      } else if (c.status.length > 0) {
        statusIds = c.status
          .map((name) => statusLookup.value.find((s) => s.name === name)?.id)
          .filter((id): id is number => id !== undefined);
      }

      const issueTypeIds = c.issueType
        ? [issueTypeLookup.value.find((t) => t.name === c.issueType)?.id].filter((id): id is number => id !== undefined)
        : undefined;

      const categoryIds = c.category
        ? [categoryLookup.value.find((cat) => cat.name === c.category)?.id].filter((id): id is number => id !== undefined)
        : undefined;

      const assigneeIds = c.assignee
        ? [userLookup.value.find((u) => u.name === c.assignee)?.id].filter((id): id is number => id !== undefined)
        : undefined;

      const priorityIds = c.priorityFilter
        ? [priorityLookup.value.find((p) => p.name === c.priorityFilter)?.id].filter((id): id is number => id !== undefined)
        : undefined;

      const result = await backlogListIssues({
        projectKey: projectId,
        count,
        offset,
        statusIds: statusIds?.length ? statusIds : undefined,
        issueTypeIds: issueTypeIds?.length ? issueTypeIds : undefined,
        categoryIds: categoryIds?.length ? categoryIds : undefined,
        assigneeIds: assigneeIds?.length ? assigneeIds : undefined,
        priorityIds: priorityIds?.length ? priorityIds : undefined,
        keyword: c.keyword || undefined,
        sort: "created",
        order: "desc",
      });

      backlogItems.value = result.issues.map(mapIssue);
      totalCount.value = result.totalCount;

      const createUserSet = new Set(backlogItems.value.map((i) => i.createUser).filter(Boolean));
      uniqueCreateUsers.value = [...createUserSet].sort();
    } catch (e) {
      searchError.value = String(e);
      backlogItems.value = [];
      totalCount.value = 0;
    } finally {
      searching.value = false;
    }
  }

  async function search() {
    appliedCriteria.value = { ...criteria.value };
    first.value = 0;

    if (!appliedCriteria.value.project) {
      backlogItems.value = [];
      totalCount.value = 0;
      searchError.value = "";
      return;
    }

    await fetchPage(0, pageSize.value);
  }

  async function onPage(event: { first: number; rows: number }) {
    first.value = event.first;
    pageSize.value = event.rows;
    await fetchPage(event.first, event.rows);
  }

  function reset() {
    criteria.value = { ...initialCriteria };
    appliedCriteria.value = { ...initialCriteria };
    backlogItems.value = [];
    totalCount.value = 0;
    first.value = 0;
    searchError.value = "";
    loadProjectLookups("");
  }

  loadProjects();

  if (initialProject) {
    resolveProjectInfo(initialProject).then(({ backlogKey }) => {
      if (backlogKey) loadProjectLookups(backlogKey);
    });
  }

  return { criteria, filteredItems, canOpenImport, setField, search, onPage, reset, lookupLoading, lookupError, searching, searchError, totalCount, first, pageSize };
}
