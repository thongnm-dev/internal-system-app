import { useMemo, useState } from "react";
import type { MessageMode } from "../types/statistics";

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

export function useProjectSkillsController() {
  const [skills, setSkills] = useState<ManagedSkill[]>(() => loadSkills());
  const [draft, setDraft] = useState<ManagedSkill>(() => loadSkills()[0] ?? createEmptySkill());
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState<SkillCategory | "All">("All");
  const [sortKey, setSortKey] = useState<SkillSortKey>("downloads");
  const [viewMode, setViewMode] = useState<SkillViewMode>("list");
  const [message, setMessage] = useState("Manage skills by name, category, usage, and operational guidance.");
  const [messageMode, setMessageMode] = useState<MessageMode>("info");

  const filteredSkills = useMemo(() => {
    const keyword = normalize(query);
    const categoryFiltered = category === "All" ? skills : skills.filter((skill) => skill.category === category);
    const searched = keyword
      ? categoryFiltered.filter((skill) =>
          normalize(
            [
              skill.name,
              skill.category,
              skill.description,
              skill.publisher,
              skill.version,
              skill.status,
              skill.tags.join(" "),
              skill.usage,
              skill.guidance,
            ].join(" "),
          ).includes(keyword),
        )
      : categoryFiltered;

    return sortSkills(searched, sortKey);
  }, [category, query, skills, sortKey]);

  const stats = useMemo<SkillCatalogStats>(
    () => ({
      active: skills.filter((skill) => skill.status === "Active").length,
      draft: skills.filter((skill) => skill.status === "Draft").length,
      total: skills.length,
    }),
    [skills],
  );

  const selectSkill = (skillId: string) => {
    const selected = skills.find((skill) => skill.id === skillId);
    if (!selected) {
      return;
    }

    setDraft(selected);
    setMessage(`Loaded ${selected.name}.`);
    setMessageMode("info");
  };

  const createSkill = () => {
    setDraft(createEmptySkill());
    setMessage("Created a new unsaved skill draft.");
    setMessageMode("info");
  };

  const updateDraft = <Field extends keyof ManagedSkill>(field: Field, value: ManagedSkill[Field]) => {
    setDraft((current) => ({ ...current, [field]: value }));
  };

  const saveDraft = () => {
    const validationError = validateDraft(draft);

    if (validationError) {
      setMessage(validationError);
      setMessageMode("error");
      return;
    }

    const nextId = slugify(draft.name);
    const duplicated = skills.some((skill) => skill.id !== draft.id && skill.id === nextId);

    if (duplicated) {
      setMessage(`A skill named ${draft.name} already exists.`);
      setMessageMode("error");
      return;
    }

    const nextDraft: ManagedSkill = {
      ...draft,
      id: nextId,
      tags: normalizeTags(draft.tags),
      updatedAt: new Date().toISOString(),
      createdAt: draft.createdAt || new Date().toISOString(),
    };

    const nextSkills = skills.some((skill) => skill.id === draft.id)
      ? skills.map((skill) => (skill.id === draft.id ? nextDraft : skill))
      : [nextDraft, ...skills];

    setSkills(nextSkills);
    setDraft(nextDraft);
    persistSkills(nextSkills);
    setMessage(`Saved ${nextDraft.name}.`);
    setMessageMode("info");
  };

  const resetDraft = () => {
    const selected = skills.find((skill) => skill.id === draft.id);

    if (!selected) {
      setDraft(createEmptySkill());
      setMessage("Cleared the unsaved skill draft.");
      setMessageMode("info");
      return;
    }

    setDraft(selected);
    setMessage(`Reverted draft changes for ${selected.name}.`);
    setMessageMode("info");
  };

  const deleteDraft = () => {
    const selected = skills.find((skill) => skill.id === draft.id);

    if (!selected) {
      setDraft(createEmptySkill());
      setMessage("Discarded the unsaved skill draft.");
      setMessageMode("info");
      return;
    }

    const nextSkills = skills.filter((skill) => skill.id !== selected.id);
    setSkills(nextSkills);
    persistSkills(nextSkills);
    setDraft(nextSkills[0] ?? createEmptySkill());
    setMessage(`Deleted ${selected.name}.`);
    setMessageMode("info");
  };

  return {
    category,
    draft,
    filteredSkills,
    generatedMarkdown: buildSkillMarkdown(draft),
    message,
    messageMode,
    query,
    selectedSkillId: draft.id,
    skills,
    sortKey,
    stats,
    viewMode,
    createSkill,
    deleteDraft,
    resetDraft,
    saveDraft,
    selectSkill,
    setCategory,
    setQuery,
    setSortKey,
    setViewMode,
    updateDraft,
  };
}

function loadSkills() {
  if (typeof window === "undefined") {
    return defaultSkills;
  }

  try {
    const value = window.localStorage.getItem(storageKey);
    if (!value) {
      return defaultSkills;
    }

    const parsed = JSON.parse(value) as ManagedSkill[];
    return parsed.length > 0 ? parsed : defaultSkills;
  } catch {
    return defaultSkills;
  }
}

function persistSkills(skills: ManagedSkill[]) {
  if (typeof window === "undefined") {
    return;
  }

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
  if (!draft.name.trim()) {
    return "Skill name is required.";
  }

  if (!draft.description.trim()) {
    return "Description is required.";
  }

  if (!draft.publisher.trim()) {
    return "Publisher is required.";
  }

  if (!draft.usage.trim()) {
    return "Usage guidance is required.";
  }

  if (!draft.guidance.trim()) {
    return "Operational guidance is required.";
  }

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

function sortSkills(skills: ManagedSkill[], sortKey: SkillSortKey) {
  return [...skills].sort((a, b) => {
    if (sortKey === "downloads") {
      return b.downloads - a.downloads;
    }
    if (sortKey === "stars") {
      return b.stars - a.stars;
    }
    if (sortKey === "installs") {
      return b.installs - a.installs;
    }
    if (sortKey === "updated") {
      return Date.parse(b.updatedAt) - Date.parse(a.updatedAt);
    }
    if (sortKey === "newest") {
      return Date.parse(b.createdAt) - Date.parse(a.createdAt);
    }
    if (sortKey === "name") {
      return a.name.localeCompare(b.name);
    }

    const statusCompare = Number(b.status === "Active") - Number(a.status === "Active");
    return statusCompare || b.installs + b.stars - (a.installs + a.stars);
  });
}

function normalizeTags(tags: string[]) {
  return Array.from(new Set(tags.map((tag) => tag.trim()).filter(Boolean)));
}

function normalize(value: string) {
  return value.trim().toLocaleLowerCase();
}

function slugify(value: string) {
  const normalized = normalize(value).replace(/[^a-z0-9]+/g, "-").replace(/(^-|-$)/g, "");
  return normalized || `skill-${Date.now()}`;
}
