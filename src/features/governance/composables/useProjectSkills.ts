import { computed, ref } from "vue";
import type { MessageMode } from "@/_/types/app";

export type SkillSortKey = "featured" | "downloads" | "stars" | "installs" | "updated" | "newest" | "name";
export type SkillViewMode = "list" | "grid";

export type SkillCategory =
  | "MCP Tools"
  | "Prompts"
  | "Workflows"
  | "Dev Tools"
  | "Data & APIs"
  | "Security"
  | "Automation"
  | "Other";

export type SkillStatus = "Active" | "Draft" | "Deprecated";

export type ManagedSkill = {
  id: string;
  name: string;
  category: SkillCategory;
  description: string;
  publisher: string;
  version: string;
  downloads: number;
  stars: number;
  installs: number;
  status: SkillStatus;
  tags: string[];
  updatedAt: string;
  createdAt: string;
  usage: string;
  guidance: string;
};

export type SkillCatalogStats = {
  active: number;
  draft: number;
  total: number;
};

export const skillCategories: SkillCategory[] = [
  "MCP Tools",
  "Prompts",
  "Workflows",
  "Dev Tools",
  "Data & APIs",
  "Security",
  "Automation",
  "Other",
];

const storageKey = "internal-system.skills-catalog.v1";

const defaultSkills: ManagedSkill[] = [
  {
    id: "browser-automation",
    name: "Browser Automation",
    category: "Automation",
    description: "Open, inspect, click, type, and verify local web screens during UI development.",
    publisher: "Internal Platform",
    version: "1.2.0",
    downloads: 1280,
    stars: 86,
    installs: 314,
    status: "Active",
    tags: ["browser", "qa", "frontend"],
    updatedAt: "2026-05-18T00:00:00.000Z",
    createdAt: "2026-03-12T00:00:00.000Z",
    usage: "Use after meaningful frontend changes or when a task asks to open or inspect localhost.",
    guidance: "Start the app, open the target route, verify layout and interaction states, then report exact issues found.",
  },
  {
    id: "github-pr-review",
    name: "GitHub PR Review",
    category: "Dev Tools",
    description: "Inspect pull requests, summarize risk, and address actionable review comments.",
    publisher: "Engineering Enablement",
    version: "2.0.1",
    downloads: 946,
    stars: 74,
    installs: 229,
    status: "Active",
    tags: ["github", "review", "ci"],
    updatedAt: "2026-05-14T00:00:00.000Z",
    createdAt: "2026-02-20T00:00:00.000Z",
    usage: "Use when work is anchored to a GitHub issue, pull request, review thread, or failing check.",
    guidance: "Fetch PR metadata first, read unresolved threads, make scoped fixes, run relevant checks, and leave concise status.",
  },
  {
    id: "spreadsheet-analysis",
    name: "Spreadsheet Analysis",
    category: "Data & APIs",
    description: "Create, edit, analyze, chart, and verify spreadsheet workbooks.",
    publisher: "Operations",
    version: "1.4.3",
    downloads: 704,
    stars: 51,
    installs: 180,
    status: "Active",
    tags: ["xlsx", "csv", "charts"],
    updatedAt: "2026-04-30T00:00:00.000Z",
    createdAt: "2026-01-22T00:00:00.000Z",
    usage: "Use for spreadsheet creation, analysis, formula updates, CSV cleanup, and workbook rendering.",
    guidance: "Preserve workbook structure, use formulas where appropriate, render key sheets, and verify values before delivery.",
  },
];

function loadSkills(): ManagedSkill[] {
  try {
    const value = window.localStorage.getItem(storageKey);
    if (!value) return defaultSkills;
    const parsed = JSON.parse(value) as ManagedSkill[];
    return parsed.length > 0 ? parsed : defaultSkills;
  } catch {
    return defaultSkills;
  }
}

function persistSkills(skills: ManagedSkill[]) {
  window.localStorage.setItem(storageKey, JSON.stringify(skills));
}

function createEmptySkill(): ManagedSkill {
  const now = new Date().toISOString();
  return {
    id: `skill-${Date.now()}`,
    name: "",
    category: "Other",
    description: "",
    publisher: "",
    version: "0.1.0",
    downloads: 0,
    stars: 0,
    installs: 0,
    status: "Draft",
    tags: [],
    updatedAt: now,
    createdAt: now,
    usage: "",
    guidance: "",
  };
}

function validateDraft(draft: ManagedSkill) {
  if (!draft.name.trim()) return "Skill name is required.";
  if (!draft.description.trim()) return "Description is required.";
  if (!draft.publisher.trim()) return "Publisher is required.";
  if (!draft.usage.trim()) return "Usage guidance is required.";
  if (!draft.guidance.trim()) return "Operational guidance is required.";
  return "";
}

function buildSkillMarkdown(skill: ManagedSkill) {
  return [
    `# ${skill.name || "Untitled Skill"}`,
    "",
    `Category: ${skill.category}`,
    `Publisher: ${skill.publisher || "-"}`,
    `Version: ${skill.version || "-"}`,
    `Status: ${skill.status}`,
    skill.tags.length ? `Tags: ${skill.tags.join(", ")}` : "Tags: -",
    "",
    "## Description",
    skill.description || "-",
    "",
    "## When To Use",
    skill.usage || "-",
    "",
    "## Guidance",
    skill.guidance || "-",
    "",
  ].join("\n");
}

function sortSkills(skills: ManagedSkill[], key: SkillSortKey) {
  return [...skills].sort((a, b) => {
    if (key === "downloads") return b.downloads - a.downloads;
    if (key === "stars") return b.stars - a.stars;
    if (key === "installs") return b.installs - a.installs;
    if (key === "updated") return Date.parse(b.updatedAt) - Date.parse(a.updatedAt);
    if (key === "newest") return Date.parse(b.createdAt) - Date.parse(a.createdAt);
    if (key === "name") return a.name.localeCompare(b.name);
    const statusCompare = Number(b.status === "Active") - Number(a.status === "Active");
    return statusCompare || b.installs + b.stars - (a.installs + a.stars);
  });
}

function normalizeTags(tags: string[]) {
  return Array.from(new Set(tags.map((t) => t.trim()).filter(Boolean)));
}

function normalize(value: string) {
  return value.trim().toLocaleLowerCase();
}

function slugify(value: string) {
  const normalized = normalize(value).replace(/[^a-z0-9]+/g, "-").replace(/(^-|-$)/g, "");
  return normalized || `skill-${Date.now()}`;
}

export function useProjectSkills() {
  const skills = ref<ManagedSkill[]>(loadSkills());
  const draft = ref<ManagedSkill>(loadSkills()[0] ?? createEmptySkill());
  const query = ref("");
  const category = ref<SkillCategory | "All">("All");
  const sortKey = ref<SkillSortKey>("downloads");
  const viewMode = ref<SkillViewMode>("list");
  const message = ref("Manage skills by name, category, usage, and operational guidance.");
  const messageMode = ref<MessageMode>("info");

  const filteredSkills = computed(() => {
    const keyword = normalize(query.value);
    const catFiltered = category.value === "All" ? skills.value : skills.value.filter((s) => s.category === category.value);
    const searched = keyword
      ? catFiltered.filter((s) =>
          normalize(
            [s.name, s.category, s.description, s.publisher, s.version, s.status, s.tags.join(" "), s.usage, s.guidance].join(" "),
          ).includes(keyword),
        )
      : catFiltered;
    return sortSkills(searched, sortKey.value);
  });

  const stats = computed<SkillCatalogStats>(() => ({
    active: skills.value.filter((s) => s.status === "Active").length,
    draft: skills.value.filter((s) => s.status === "Draft").length,
    total: skills.value.length,
  }));

  const generatedMarkdown = computed(() => buildSkillMarkdown(draft.value));
  const selectedSkillId = computed(() => draft.value.id);

  function selectSkill(skillId: string) {
    const selected = skills.value.find((s) => s.id === skillId);
    if (!selected) return;
    draft.value = { ...selected };
    message.value = `Loaded ${selected.name}.`;
    messageMode.value = "info";
  }

  function createSkill() {
    draft.value = createEmptySkill();
    message.value = "Created a new unsaved skill draft.";
    messageMode.value = "info";
  }

  function updateDraft<Field extends keyof ManagedSkill>(field: Field, value: ManagedSkill[Field]) {
    (draft.value as any)[field] = value;
  }

  function saveDraft() {
    const validationError = validateDraft(draft.value);
    if (validationError) {
      message.value = validationError;
      messageMode.value = "error";
      return;
    }

    const nextId = slugify(draft.value.name);
    const duplicated = skills.value.some((s) => s.id !== draft.value.id && s.id === nextId);
    if (duplicated) {
      message.value = `A skill named ${draft.value.name} already exists.`;
      messageMode.value = "error";
      return;
    }

    const nextDraft: ManagedSkill = {
      ...draft.value,
      id: nextId,
      tags: normalizeTags(draft.value.tags),
      updatedAt: new Date().toISOString(),
      createdAt: draft.value.createdAt || new Date().toISOString(),
    };

    const nextSkills = skills.value.some((s) => s.id === draft.value.id)
      ? skills.value.map((s) => (s.id === draft.value.id ? nextDraft : s))
      : [nextDraft, ...skills.value];

    skills.value = nextSkills;
    draft.value = { ...nextDraft };
    persistSkills(nextSkills);
    message.value = `Saved ${nextDraft.name}.`;
    messageMode.value = "info";
  }

  function resetDraft() {
    const selected = skills.value.find((s) => s.id === draft.value.id);
    if (!selected) {
      draft.value = createEmptySkill();
      message.value = "Cleared the unsaved skill draft.";
      messageMode.value = "info";
      return;
    }
    draft.value = { ...selected };
    message.value = `Reverted draft changes for ${selected.name}.`;
    messageMode.value = "info";
  }

  function deleteDraft() {
    const selected = skills.value.find((s) => s.id === draft.value.id);
    if (!selected) {
      draft.value = createEmptySkill();
      message.value = "Discarded the unsaved skill draft.";
      messageMode.value = "info";
      return;
    }

    const nextSkills = skills.value.filter((s) => s.id !== selected.id);
    skills.value = nextSkills;
    persistSkills(nextSkills);
    draft.value = nextSkills[0] ? { ...nextSkills[0] } : createEmptySkill();
    message.value = `Deleted ${selected.name}.`;
    messageMode.value = "info";
  }

  return {
    category,
    draft,
    filteredSkills,
    generatedMarkdown,
    message,
    messageMode,
    query,
    selectedSkillId,
    skills,
    sortKey,
    stats,
    viewMode,
    createSkill,
    deleteDraft,
    resetDraft,
    saveDraft,
    selectSkill,
    updateDraft,
  };
}
