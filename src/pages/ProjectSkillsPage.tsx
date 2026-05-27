import { Copy, Download, Grid2X2, List, Plus, RotateCcw, Save, Search, Star, Trash2 } from "lucide-react";
import { Button } from "primereact/button";
import { Dropdown } from "primereact/dropdown";
import { Fieldset } from "primereact/fieldset";
import { InputText } from "primereact/inputtext";
import { SelectButton } from "primereact/selectbutton";
import { useState } from "react";
import { MessageBanner } from "../components/MessageBanner";
import {
  skillCategories,
  type ManagedSkill,
  type SkillCatalogStats,
  type SkillCategory,
  type SkillSortKey,
  type SkillStatus,
  type SkillViewMode,
} from "../controller/useProjectSkillsController";
import type { MessageMode } from "../types/statistics";

type ProjectSkillsPageProps = {
  category: SkillCategory | "All";
  draft: ManagedSkill;
  filteredSkills: ManagedSkill[];
  generatedMarkdown: string;
  message: string;
  messageMode: MessageMode;
  query: string;
  selectedSkillId: string;
  sortKey: SkillSortKey;
  stats: SkillCatalogStats;
  viewMode: SkillViewMode;
  onCategoryChange: (value: SkillCategory | "All") => void;
  onCreate: () => void;
  onDelete: () => void;
  onQueryChange: (value: string) => void;
  onReset: () => void;
  onSave: () => void;
  onSelectSkill: (skillId: string) => void;
  onSortChange: (value: SkillSortKey) => void;
  onUpdateDraft: <Field extends keyof ManagedSkill>(field: Field, value: ManagedSkill[Field]) => void;
  onViewModeChange: (value: SkillViewMode) => void;
};

const sortOptions: { label: string; value: SkillSortKey }[] = [
  { label: "Featured", value: "featured" },
  { label: "Most downloaded", value: "downloads" },
  { label: "Most starred", value: "stars" },
  { label: "Most installed", value: "installs" },
  { label: "Recently updated", value: "updated" },
  { label: "Newest", value: "newest" },
  { label: "Name", value: "name" },
];

const statusOptions: SkillStatus[] = ["Active", "Draft", "Deprecated"];
const categoryOptions = ["All", ...skillCategories] as const;

export function ProjectSkillsPage({
  category,
  draft,
  filteredSkills,
  generatedMarkdown,
  message,
  messageMode,
  query,
  selectedSkillId,
  sortKey,
  stats,
  viewMode,
  onCategoryChange,
  onCreate,
  onDelete,
  onQueryChange,
  onReset,
  onSave,
  onSelectSkill,
  onSortChange,
  onUpdateDraft,
  onViewModeChange,
}: ProjectSkillsPageProps) {
  const [copyMessage, setCopyMessage] = useState("");

  const copyMarkdown = async () => {
    try {
      await navigator.clipboard.writeText(generatedMarkdown);
      setCopyMessage("Generated skill markdown copied to clipboard.");
    } catch {
      setCopyMessage("Clipboard is unavailable in this runtime.");
    }
  };

  const confirmDelete = () => {
    const skillLabel = draft.name || "this unsaved skill";
    if (window.confirm(`Delete ${skillLabel}?`)) {
      onDelete();
    }
  };

  const updateTags = (value: string) => {
    onUpdateDraft(
      "tags",
      value
        .split(",")
        .map((tag) => tag.trim())
        .filter(Boolean),
    );
  };

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <section className="grid gap-3 rounded-lg border border-stone-200 bg-panel p-4 shadow-sm xl:grid-cols-[minmax(260px,1fr)_auto]">
        <div className="grid gap-3 lg:grid-cols-[minmax(220px,1fr)_220px]">
          <label className="block min-w-0">
            <span className="text-xs font-bold text-slate-500">Search Skills</span>
            <span className="mt-1 flex h-10 items-center gap-2 rounded-md border border-slate-300 bg-white px-3 focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100">
              <Search className="h-4 w-4 shrink-0 text-slate-400" />
              <InputText
                className="min-w-0 flex-1 border-0 bg-transparent p-0 text-sm text-slate-900 outline-none shadow-none"
                placeholder="Name, tag, category, guidance"
                type="search"
                value={query}
                onChange={(event) => onQueryChange(event.target.value)}
              />
            </span>
          </label>
          <label className="block min-w-0">
            <span className="text-xs font-bold text-slate-500">Sort by</span>
            <Dropdown
              className="mt-1 flex h-10 items-center rounded-md border border-slate-300 bg-white text-sm"
              value={sortKey}
              options={sortOptions}
              onChange={(event) => onSortChange(event.value)}
            />
          </label>
        </div>

        <div className="flex flex-wrap items-end justify-between gap-2 xl:justify-end">
          <Metric label="Skills" value={stats.total} />
          <Metric label="Active" value={stats.active} />
          <Metric label="Drafts" value={stats.draft} />
          <Button
            className="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90"
            type="button"
            title="Create skill"
            onClick={onCreate}
          >
            <Plus className="h-4 w-4" />
            New
          </Button>
        </div>
      </section>

      <section className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap gap-2">
          {categoryOptions.map((item) => (
            <Button
              key={item}
              className={[
                "h-9 rounded-md border px-3 text-xs font-bold",
                category === item
                  ? "border-brand bg-emerald-50 text-brand"
                  : "border-slate-300 bg-white text-slate-600 hover:bg-slate-50",
              ].join(" ")}
              type="button"
              onClick={() => onCategoryChange(item)}
            >
              {item}
            </Button>
          ))}
        </div>
        <SelectButton
          className="grid h-9 grid-cols-2 gap-1 rounded-md border border-slate-300 bg-white p-1 [&_.p-button]:h-7 [&_.p-button]:px-2"
          value={viewMode}
          options={[
            { label: <List className="h-4 w-4" />, value: "list" },
            { label: <Grid2X2 className="h-4 w-4" />, value: "grid" },
          ]}
          allowEmpty={false}
          onChange={(event) => {
            if (event.value) {
              onViewModeChange(event.value);
            }
          }}
        />
      </section>

      <MessageBanner message={message} mode={messageMode} />

      <section className="grid min-h-0 flex-1 gap-4 overflow-hidden xl:grid-cols-[minmax(360px,1fr)_minmax(420px,520px)]">
        <section className="min-h-0 overflow-auto rounded-lg border border-stone-200 bg-panel p-3 shadow-sm">
          {filteredSkills.length === 0 ? (
            <p className="p-3 text-sm text-slate-500">No skills match the current filters.</p>
          ) : (
            <div className={viewMode === "grid" ? "grid gap-3 2xl:grid-cols-2" : "grid gap-2"}>
              {filteredSkills.map((skill) => (
                <SkillCard
                  key={skill.id}
                  isActive={selectedSkillId === skill.id}
                  skill={skill}
                  viewMode={viewMode}
                  onClick={() => onSelectSkill(skill.id)}
                />
              ))}
            </div>
          )}
        </section>

        <section className="flex min-h-0 flex-col overflow-hidden rounded-lg border border-stone-200 bg-panel shadow-sm">
          <div className="flex items-center justify-between gap-3 border-b border-stone-200 px-4 py-3">
            <h3 className="font-bold">Skill Details</h3>
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
                title="Save skill"
                onClick={onSave}
              >
                <Save className="h-4 w-4" />
                Save
              </Button>
            </div>
          </div>

          <div className="min-h-0 flex-1 overflow-auto p-4">
            <Fieldset
              className="rounded-lg border border-slate-200 bg-white p-4 shadow-md fieldset-nested"
              legend="Metadata"
              toggleable
            >
              <div className="grid gap-3 md:grid-cols-2">
                <label className="block min-w-0 md:col-span-2">
                  <span className="text-xs font-bold text-slate-500">Skill Name</span>
                  <InputText
                    className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                    placeholder="Skill name"
                    value={draft.name}
                    onChange={(event) => onUpdateDraft("name", event.target.value)}
                  />
                </label>
                <label className="block min-w-0">
                  <span className="text-xs font-bold text-slate-500">Category</span>
                  <Dropdown
                    className="mt-1 flex h-10 items-center rounded-md border border-slate-300 bg-white text-sm"
                    value={draft.category}
                    options={skillCategories}
                    onChange={(event) => onUpdateDraft("category", event.value)}
                  />
                </label>
                <label className="block min-w-0">
                  <span className="text-xs font-bold text-slate-500">Status</span>
                  <Dropdown
                    className="mt-1 flex h-10 items-center rounded-md border border-slate-300 bg-white text-sm"
                    value={draft.status}
                    options={statusOptions}
                    onChange={(event) => onUpdateDraft("status", event.value)}
                  />
                </label>
                <label className="block min-w-0">
                  <span className="text-xs font-bold text-slate-500">Publisher</span>
                  <InputText
                    className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                    placeholder="Publisher"
                    value={draft.publisher}
                    onChange={(event) => onUpdateDraft("publisher", event.target.value)}
                  />
                </label>
                <label className="block min-w-0">
                  <span className="text-xs font-bold text-slate-500">Version</span>
                  <InputText
                    className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                    placeholder="0.1.0"
                    value={draft.version}
                    onChange={(event) => onUpdateDraft("version", event.target.value)}
                  />
                </label>
                <label className="block min-w-0 md:col-span-2">
                  <span className="text-xs font-bold text-slate-500">Description</span>
                  <textarea
                    className="mt-1 min-h-20 w-full resize-y rounded-md border border-slate-300 bg-white px-3 py-2 text-sm leading-6 text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                    placeholder="What this skill does"
                    value={draft.description}
                    onChange={(event) => onUpdateDraft("description", event.target.value)}
                  />
                </label>
                <label className="block min-w-0 md:col-span-2">
                  <span className="text-xs font-bold text-slate-500">Tags</span>
                  <InputText
                    className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                    placeholder="browser, qa, frontend"
                    value={draft.tags.join(", ")}
                    onChange={(event) => updateTags(event.target.value)}
                  />
                </label>
              </div>
            </Fieldset>

            <Fieldset
              className="mt-4 rounded-lg border border-slate-200 bg-white p-4 shadow-md fieldset-nested"
              legend="Guidance"
              toggleable
            >
              <label className="block min-w-0">
                <span className="text-xs font-bold text-slate-500">When To Use</span>
                <textarea
                  className="mt-1 min-h-24 w-full resize-y rounded-md border border-slate-300 bg-white px-3 py-2 text-sm leading-6 text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  placeholder="Use when..."
                  value={draft.usage}
                  onChange={(event) => onUpdateDraft("usage", event.target.value)}
                />
              </label>
              <label className="mt-3 block min-w-0">
                <span className="text-xs font-bold text-slate-500">Operational Guidance</span>
                <textarea
                  className="mt-1 min-h-32 w-full resize-y rounded-md border border-slate-300 bg-white px-3 py-2 text-sm leading-6 text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  placeholder="Step-by-step behavior, constraints, and verification"
                  value={draft.guidance}
                  onChange={(event) => onUpdateDraft("guidance", event.target.value)}
                />
              </label>
            </Fieldset>

            <Fieldset
              className="mt-4 rounded-lg border border-slate-200 bg-white p-4 shadow-md fieldset-nested"
              legend="Catalog Metrics"
              toggleable
              collapsed
            >
              <div className="grid gap-3 md:grid-cols-3">
                <NumberField label="Downloads" value={draft.downloads} onChange={(value) => onUpdateDraft("downloads", value)} />
                <NumberField label="Stars" value={draft.stars} onChange={(value) => onUpdateDraft("stars", value)} />
                <NumberField label="Installs" value={draft.installs} onChange={(value) => onUpdateDraft("installs", value)} />
              </div>
            </Fieldset>

            <section className="mt-4 flex min-h-0 flex-col overflow-hidden rounded-lg border border-stone-200 bg-white">
              <div className="flex items-center justify-between gap-3 border-b border-stone-200 px-4 py-3">
                <h3 className="font-bold">Generated Markdown</h3>
                <Button
                  className="flex h-9 items-center gap-2 rounded-md border border-slate-300 bg-white px-3 text-sm font-bold text-slate-600 hover:bg-slate-50"
                  type="button"
                  title="Copy generated markdown"
                  onClick={() => void copyMarkdown()}
                >
                  <Copy className="h-4 w-4" />
                  Copy
                </Button>
              </div>
              {copyMessage ? <p className="border-b border-stone-200 px-4 py-2 text-xs text-slate-500">{copyMessage}</p> : null}
              <pre className="max-h-72 overflow-auto whitespace-pre-wrap p-4 font-mono text-xs leading-5 text-slate-800">
                {generatedMarkdown}
              </pre>
            </section>

            <div className="mt-4 flex justify-end">
              <Button
                className="flex h-9 items-center gap-2 rounded-md border border-red-200 bg-white px-3 text-sm font-bold text-red-700 hover:bg-red-50"
                type="button"
                title="Delete skill"
                onClick={confirmDelete}
              >
                <Trash2 className="h-4 w-4" />
                Delete
              </Button>
            </div>
          </div>
        </section>
      </section>
    </section>
  );
}

function Metric({ label, value }: { label: string; value: number }) {
  return (
    <span className="min-w-20 rounded-md border border-slate-200 bg-white px-3 py-2 text-right">
      <span className="block text-[11px] font-bold uppercase text-slate-500">{label}</span>
      <span className="block text-sm font-extrabold text-slate-900">{value.toLocaleString("en-US")}</span>
    </span>
  );
}

function SkillCard({
  isActive,
  skill,
  viewMode,
  onClick,
}: {
  isActive: boolean;
  skill: ManagedSkill;
  viewMode: SkillViewMode;
  onClick: () => void;
}) {
  return (
    <Button
      className={[
        "w-full rounded-md border bg-white p-4 text-left transition",
        isActive ? "border-brand ring-2 ring-emerald-100" : "border-slate-200 hover:bg-slate-50",
        viewMode === "list" ? "flex items-start justify-between gap-4" : "block min-h-52",
      ].join(" ")}
      type="button"
      onClick={onClick}
    >
      <span className="min-w-0 flex-1">
        <span className="flex min-w-0 flex-wrap items-center gap-2">
          <span className="min-w-0 truncate text-base font-extrabold text-slate-900">{skill.name || "Untitled Skill"}</span>
          <span className="shrink-0 rounded-md border border-slate-200 px-2 py-0.5 text-[11px] font-bold text-slate-500">
            {skill.category}
          </span>
          <span
            className={[
              "shrink-0 rounded-md px-2 py-0.5 text-[11px] font-bold",
              skill.status === "Active"
                ? "bg-emerald-50 text-emerald-700"
                : skill.status === "Deprecated"
                  ? "bg-red-50 text-red-700"
                  : "bg-amber-50 text-amber-700",
            ].join(" ")}
          >
            {skill.status}
          </span>
        </span>
        <span className="mt-2 block text-sm font-normal leading-6 text-slate-600">{skill.description || "No description"}</span>
        <span className="mt-3 flex flex-wrap gap-1">
          {skill.tags.slice(0, 4).map((tag) => (
            <span key={tag} className="rounded-md bg-slate-100 px-2 py-1 text-[11px] font-bold text-slate-500">
              {tag}
            </span>
          ))}
        </span>
      </span>
      <span className={viewMode === "list" ? "grid shrink-0 gap-2 text-right" : "mt-4 flex gap-4"}>
        <span className="flex items-center justify-end gap-1 text-xs font-bold text-slate-500">
          <Download className="h-3.5 w-3.5" />
          {skill.downloads.toLocaleString("en-US")}
        </span>
        <span className="flex items-center justify-end gap-1 text-xs font-bold text-slate-500">
          <Star className="h-3.5 w-3.5" />
          {skill.stars.toLocaleString("en-US")}
        </span>
      </span>
    </Button>
  );
}

function NumberField({ label, value, onChange }: { label: string; value: number; onChange: (value: number) => void }) {
  return (
    <label className="block min-w-0">
      <span className="text-xs font-bold text-slate-500">{label}</span>
      <InputText
        className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
        inputMode="numeric"
        value={String(value)}
        onChange={(event) => onChange(Math.max(0, Number(event.target.value) || 0))}
      />
    </label>
  );
}
