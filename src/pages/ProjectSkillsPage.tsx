import { ArrowLeft, Copy, RotateCcw, Save } from "lucide-react";
import { Button } from "primereact/button";
import { Fieldset } from "primereact/fieldset";
import { InputText } from "primereact/inputtext";
import { SelectButton } from "primereact/selectbutton";
import { useState } from "react";
import { MessageBanner } from "../components/MessageBanner";
import type { ProjectSkill, SkillRoleKey } from "../controller/useProjectSkillsController";
import type { MessageMode } from "../types/statistics";

type ProjectSkillsPageProps = {
  draft: ProjectSkill;
  generatedMarkdown: string;
  message: string;
  messageMode: MessageMode;
  onBack: () => void;
  onReset: () => void;
  onSave: () => void;
  onUpdateDraft: <Field extends keyof ProjectSkill>(field: Field, value: ProjectSkill[Field]) => void;
  onUpdateRole: (role: SkillRoleKey, value: string) => void;
};

const roleOptions: { label: string; value: SkillRoleKey }[] = [
  { label: "Technical Architecture", value: "technicalArchitecture" },
  { label: "Coding Rule", value: "codingRule" },
  { label: "Review", value: "review" },
];

const roleLabels: Record<SkillRoleKey, string> = {
  technicalArchitecture: "Technical Architecture",
  codingRule: "Coding Rule",
  review: "Review",
};

const rolePlaceholders: Record<SkillRoleKey, string> = {
  technicalArchitecture: "Architecture boundaries, data flow, module responsibility, integration decisions...",
  codingRule: "Naming, folder patterns, framework rules, error handling, tests, formatting...",
  review: "Review checklist, risk areas, acceptance criteria, regression checks...",
};

export function ProjectSkillsPage({
  draft,
  generatedMarkdown,
  message,
  messageMode,
  onBack,
  onReset,
  onSave,
  onUpdateDraft,
  onUpdateRole,
}: ProjectSkillsPageProps) {
  const [activeRole, setActiveRole] = useState<SkillRoleKey>("technicalArchitecture");
  const [copyMessage, setCopyMessage] = useState("");

  const copyMarkdown = async () => {
    try {
      await navigator.clipboard.writeText(generatedMarkdown);
      setCopyMessage("Generated SKILL.md copied to clipboard.");
    } catch {
      setCopyMessage("Clipboard is unavailable in this runtime.");
    }
  };

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <div className="flex items-center justify-between gap-3">
        <Button
          className="flex h-9 items-center gap-2 rounded-md border border-slate-300 bg-white px-3 text-sm font-bold text-slate-700 hover:bg-slate-50"
          type="button"
          onClick={onBack}
        >
          <ArrowLeft className="h-4 w-4" />
          Back
        </Button>
        <div className="flex items-center gap-2">
          <Button
            className="flex h-9 items-center gap-2 rounded-md border border-slate-300 bg-white px-3 text-sm font-bold text-slate-600 hover:bg-slate-50"
            type="button"
            title="Reset draft"
            onClick={onReset}
          >
            <RotateCcw className="h-4 w-4" />
            Reset
          </Button>
          <Button
            className="flex h-9 items-center gap-2 rounded-md bg-brand px-3 text-sm font-bold text-white hover:opacity-90"
            type="button"
            title="Save SKILL.md guidance"
            onClick={onSave}
          >
            <Save className="h-4 w-4" />
            Save
          </Button>
        </div>
      </div>

      <MessageBanner message={message} mode={messageMode} />

      <Fieldset
        className="rounded-lg border border-slate-200 bg-white p-4 shadow-md fieldset-nested"
        legend="Project Information"
        toggleable
      >
        <div className="grid gap-3 lg:grid-cols-[minmax(260px,360px)_minmax(0,1fr)]">
          <label className="block min-w-0">
            <span className="text-xs font-bold text-slate-500">Name</span>
            <InputText
              className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Project name"
              value={draft.projectName}
              onChange={(event) => onUpdateDraft("projectName", event.target.value)}
            />
          </label>
          <label className="block min-w-0">
            <span className="text-xs font-bold text-slate-500">Description</span>
            <InputText
              className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Technology stack used in this project"
              value={draft.techStack}
              onChange={(event) => onUpdateDraft("techStack", event.target.value)}
            />
          </label>
        </div>
      </Fieldset>

      <section className="grid min-h-0 flex-1 gap-4 overflow-hidden xl:grid-cols-[minmax(0,1fr)_420px]">
        <section className="min-h-0 overflow-auto rounded-lg border border-stone-200 bg-panel p-4 shadow-sm">
          <h3 className="font-bold">Skill roles</h3>

          <div className="mt-4">
            <span className="text-xs font-bold text-slate-500">SKILL Role</span>
            <SelectButton
              className="mt-1 grid min-h-10 min-w-0 grid-cols-3 gap-1 rounded-md border border-slate-300 bg-white p-1 text-xs leading-none [&_.p-button]:min-w-0 [&_.p-button]:justify-center [&_.p-button]:px-2 [&_.p-button]:py-2 [&_.p-button-label]:truncate [&_.p-button-label]:text-xs [&_.p-button-label]:font-bold"
              value={activeRole}
              options={roleOptions}
              allowEmpty={false}
              onChange={(event) => {
                if (event.value) {
                  setActiveRole(event.value);
                }
              }}
            />
          </div>

          <label className="mt-4 block min-w-0">
            <span className="text-xs font-bold text-slate-500">{roleLabels[activeRole]}</span>
            <textarea
              className="mt-1 min-h-[300px] w-full resize-y rounded-md border border-slate-300 bg-white px-3 py-2 text-sm leading-6 text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder={rolePlaceholders[activeRole]}
              value={draft.roles[activeRole]}
              onChange={(event) => onUpdateRole(activeRole, event.target.value)}
            />
          </label>

          <div className="mt-4 grid gap-3 md:grid-cols-3">
            {roleOptions.map((role) => (
              <RoleStatus
                key={role.value}
                label={role.label}
                isComplete={draft.roles[role.value].trim().length > 0}
                onClick={() => setActiveRole(role.value)}
              />
            ))}
          </div>
        </section>

        <section className="flex min-h-0 flex-col overflow-hidden rounded-lg border border-stone-200 bg-panel shadow-sm">
          <div className="flex items-center justify-between gap-3 border-b border-stone-200 px-4 py-3">
            <h3 className="font-bold">Generated SKILL.md</h3>
            <Button
              className="flex h-9 items-center gap-2 rounded-md border border-slate-300 bg-white px-3 text-sm font-bold text-slate-600 hover:bg-slate-50"
              type="button"
              title="Copy generated SKILL.md"
              onClick={() => void copyMarkdown()}
            >
              <Copy className="h-4 w-4" />
              Copy
            </Button>
          </div>
          {copyMessage ? <p className="border-b border-stone-200 px-4 py-2 text-xs text-slate-500">{copyMessage}</p> : null}
          <pre className="min-h-0 flex-1 overflow-auto whitespace-pre-wrap p-4 font-mono text-xs leading-5 text-slate-800">
            {generatedMarkdown}
          </pre>
        </section>
      </section>
    </section>
  );
}

function RoleStatus({ isComplete, label, onClick }: { isComplete: boolean; label: string; onClick: () => void }) {
  return (
    <Button
      className={[
        "flex min-h-11 items-center justify-between gap-2 rounded-md border px-3 text-left text-xs font-bold",
        isComplete
          ? "border-emerald-200 bg-emerald-50 text-emerald-800"
          : "border-amber-200 bg-amber-50 text-amber-800",
      ].join(" ")}
      type="button"
      onClick={onClick}
    >
      <span className="min-w-0 truncate">{label}</span>
      <span className="shrink-0">{isComplete ? "Ready" : "Missing"}</span>
    </Button>
  );
}
