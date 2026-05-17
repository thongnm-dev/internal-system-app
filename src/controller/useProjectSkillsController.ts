import { useState } from "react";
import type { MessageMode } from "../types/statistics";

export type SkillRoleKey = "technicalArchitecture" | "codingRule" | "review";

export type ProjectSkill = {
  id: string;
  projectCode: string;
  projectName: string;
  techStack: string;
  updatedAt: string;
  roles: Record<SkillRoleKey, string>;
};

const storageKey = "internal-system.project-skills.v1";

const defaultProjectSkills: ProjectSkill[] = [
  {
    id: "internal-system-app",
    projectCode: "INT-SYS",
    projectName: "Internal System App",
    techStack: "React, Tauri, Rust",
    updatedAt: "2026-05-17T00:00:00.000Z",
    roles: {
      technicalArchitecture:
        "Define the React/Vite frontend boundary, Tauri command boundary, Rust domain services, and persistence responsibilities before implementation.",
      codingRule:
        "Follow existing page/controller/component patterns. Keep business logic in controllers or Rust domain services, and keep UI components focused on rendering and user interaction.",
      review:
        "Review route coverage, Tauri command error handling, state persistence, and UX regressions across desktop-sized layouts.",
    },
  },
  {
    id: "reporting-hub",
    projectCode: "REP-HUB",
    projectName: "Reporting Hub",
    techStack: "React, Node.js, PostgreSQL",
    updatedAt: "2026-05-17T00:00:00.000Z",
    roles: {
      technicalArchitecture:
        "Model report ingestion, aggregation, and dashboard APIs separately. Keep data contracts explicit and versioned.",
      codingRule:
        "Prefer typed query helpers, small API handlers, and deterministic date calculations for all reporting logic.",
      review:
        "Check aggregation correctness, timezone behavior, permission boundaries, and query performance before approval.",
    },
  },
];

const emptyRoles: Record<SkillRoleKey, string> = {
  technicalArchitecture: "",
  codingRule: "",
  review: "",
};

export function useProjectSkillsController(projectCode: string, projectName = "") {
  const [skills, setSkills] = useState<ProjectSkill[]>(() => loadProjectSkills());
  const [draft, setDraft] = useState<ProjectSkill>(() => findOrCreateProjectSkill(loadProjectSkills(), projectCode, projectName));
  const [message, setMessage] = useState("Edit each SKILL role, then save the generated guidance.");
  const [messageMode, setMessageMode] = useState<MessageMode>("info");

  const updateDraft = <Field extends keyof ProjectSkill>(field: Field, value: ProjectSkill[Field]) => {
    setDraft((current) => ({ ...current, [field]: value }));
  };

  const updateRole = (role: SkillRoleKey, value: string) => {
    setDraft((current) => ({
      ...current,
      roles: {
        ...current.roles,
        [role]: value,
      },
    }));
  };

  const saveDraft = () => {
    const validationError = validateDraft(draft);

    if (validationError) {
      setMessage(validationError);
      setMessageMode("error");
      return;
    }

    const nextDraft: ProjectSkill = {
      ...draft,
      id: draft.id || slugify(draft.projectCode),
      updatedAt: new Date().toISOString(),
    };

    const nextSkills = skills.some((skill) => skill.id === nextDraft.id)
      ? skills.map((skill) => (skill.id === nextDraft.id ? nextDraft : skill))
      : [nextDraft, ...skills];

    setSkills(nextSkills);
    setDraft(nextDraft);
    persistProjectSkills(nextSkills);
    setMessage(`Saved SKILL.md guidance for ${nextDraft.projectCode}.`);
    setMessageMode("info");
  };

  const resetDraft = () => {
    const selected = skills.find((skill) => skill.id === draft.id);

    if (!selected) {
      return;
    }

    setDraft(selected);
    setMessage(`Reverted draft changes for ${selected.projectCode}.`);
    setMessageMode("info");
  };

  return {
    draft,
    generatedMarkdown: buildSkillMarkdown(draft),
    message,
    messageMode,
    resetDraft,
    saveDraft,
    updateDraft,
    updateRole,
  };
}

function loadProjectSkills() {
  if (typeof window === "undefined") {
    return defaultProjectSkills;
  }

  try {
    const value = window.localStorage.getItem(storageKey);
    if (!value) {
      return defaultProjectSkills;
    }

    const parsed = JSON.parse(value) as ProjectSkill[];
    return parsed.length > 0 ? parsed : defaultProjectSkills;
  } catch {
    return defaultProjectSkills;
  }
}

function persistProjectSkills(skills: ProjectSkill[]) {
  if (typeof window === "undefined") {
    return;
  }

  window.localStorage.setItem(storageKey, JSON.stringify(skills));
}

function findOrCreateProjectSkill(skills: ProjectSkill[], projectCode: string, projectName: string): ProjectSkill {
  const id = slugify(projectCode);
  const existing = skills.find((skill) => skill.id === id || skill.projectCode === projectCode);
  if (existing) {
    return existing;
  }

  return {
    id,
    projectCode,
    projectName,
    techStack: "",
    updatedAt: new Date().toISOString(),
    roles: { ...emptyRoles },
  };
}

function validateDraft(draft: ProjectSkill) {
  if (!draft.projectCode.trim()) {
    return "Project Code is required.";
  }

  if (!draft.projectName.trim()) {
    return "Project Name is required.";
  }

  if (!draft.techStack.trim()) {
    return "Tech Stack is required.";
  }

  if (!draft.roles.technicalArchitecture.trim() || !draft.roles.codingRule.trim() || !draft.roles.review.trim()) {
    return "All three SKILL roles must be filled before saving.";
  }

  return "";
}

function buildSkillMarkdown(skill: ProjectSkill) {
  return [
    `# ${skill.projectCode || "PROJECT"} SKILL.md`,
    "",
    `Project: ${skill.projectName || "-"}`,
    `Tech stack: ${skill.techStack || "-"}`,
    "",
    "## Technical Architecture",
    skill.roles.technicalArchitecture || "-",
    "",
    "## Coding Rule",
    skill.roles.codingRule || "-",
    "",
    "## Review",
    skill.roles.review || "-",
    "",
  ].join("\n");
}

function normalize(value: string) {
  return value.trim().toLocaleLowerCase();
}

function slugify(value: string) {
  const normalized = normalize(value).replace(/[^a-z0-9]+/g, "-").replace(/(^-|-$)/g, "");
  return normalized || `project-skill-${Date.now()}`;
}
