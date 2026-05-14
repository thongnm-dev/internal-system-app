import { CalendarDays, ChevronLeft, ChevronRight, Clock, Plus } from "lucide-react";
import { useState } from "react";
import type { DailyReportEntries, DailyReportEntry, DailyReportProject, DailyReportTask } from "../controller/useDailyReportController";
import { entryHour, entryKey } from "../controller/useDailyReportController";

type DailyReportDay = {
  day: number;
  label: string;
  weekday: string;
  isWeekend: boolean;
  isToday: boolean;
};

type DailyReportPageProps = {
  availableProjects: DailyReportProject[];
  canGoNextMonth: boolean;
  days: DailyReportDay[];
  entries: DailyReportEntries;
  maxMonthValue: string;
  monthLabel: string;
  monthValue: string;
  onAddProject: (projectId: string) => void;
  onNextMonth: () => void;
  onPreviousMonth: () => void;
  onSelectMonth: (value: string) => void;
  onUpdateEntry: (taskId: string, day: number, value: DailyReportEntry | null) => void;
  projects: DailyReportProject[];
  totalHours: (taskId?: string) => number;
};

const dayCellClass = "flex h-14 w-12 shrink-0 items-center justify-center border-r border-stone-200 px-1";
const projectRowClass = "flex h-12 items-center border-b border-emerald-200 bg-[#4cbd9b] px-4";
const taskRowClass = "flex h-14 items-center border-b border-stone-200 bg-white px-4";

export function DailyReportPage({
  availableProjects,
  canGoNextMonth,
  days,
  entries,
  maxMonthValue,
  monthLabel,
  monthValue,
  onAddProject,
  onNextMonth,
  onPreviousMonth,
  onSelectMonth,
  onUpdateEntry,
  projects,
  totalHours,
}: DailyReportPageProps) {
  const [isAddingProject, setIsAddingProject] = useState(false);
  const [editingCell, setEditingCell] = useState<EditingCell | null>(null);

  return (
    <section className="grid min-h-0 flex-1 grid-cols-[380px_minmax(0,1fr)] overflow-hidden rounded-lg border border-stone-200 bg-panel shadow-sm">
      <section className="flex h-[76px] items-center justify-between gap-3 border-b border-r border-stone-200 bg-white px-4">
        <div className="min-w-0">
          <h3 className="truncate font-bold">Assigned work</h3>
          <p className="mt-1 truncate text-xs text-slate-500">{projects.length.toLocaleString("en-US")} projects</p>
        </div>
        <div className="relative">
          <button
            className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-brand text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
            type="button"
            title="Add project"
            disabled={availableProjects.length === 0}
            onClick={() => setIsAddingProject((current) => !current)}
          >
            <Plus className="h-4 w-4" />
          </button>

          {isAddingProject ? (
            <div className="absolute right-0 top-11 z-40 w-72 overflow-hidden rounded-md border border-slate-200 bg-white shadow-lg">
              {availableProjects.length === 0 ? (
                <p className="px-3 py-2 text-sm text-slate-500">No more projects.</p>
              ) : (
                availableProjects.map((project) => (
                  <button
                    key={project.id}
                    className="flex w-full items-center justify-between gap-2 border-b border-slate-200 px-3 py-2 text-left text-sm last:border-b-0 hover:bg-slate-50"
                    type="button"
                    onClick={() => {
                      onAddProject(project.id);
                      setIsAddingProject(false);
                    }}
                  >
                    <span className="min-w-0">
                      <strong className="block truncate text-slate-800">{project.code}</strong>
                      <span className="block truncate text-xs text-slate-500">{project.name}</span>
                    </span>
                    <Plus className="h-4 w-4 shrink-0 text-brand" />
                  </button>
                ))
              )}
            </div>
          ) : null}
        </div>
      </section>

      <section className="flex h-[76px] items-center justify-between gap-4 border-b border-stone-200 bg-white px-4">
        <div className="flex min-w-0 items-center gap-3">
          <span className="flex h-10 w-10 items-center justify-center rounded-md bg-emerald-50 text-brand">
            <CalendarDays className="h-5 w-5" />
          </span>
          <div className="min-w-0">
            <h3 className="truncate font-bold">{monthLabel}</h3>
            <p className="mt-1 truncate text-xs text-slate-500">Daily work hour input</p>
          </div>
        </div>
        <div className="flex shrink-0 items-center gap-2">
          <button
            className="flex h-9 w-9 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-600 hover:bg-slate-50"
            type="button"
            title="Previous month"
            onClick={onPreviousMonth}
          >
            <ChevronLeft className="h-4 w-4" />
          </button>
          <input
            className="h-9 rounded-md border border-slate-300 bg-white px-3 text-sm font-bold text-slate-700 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            max={maxMonthValue}
            type="month"
            value={monthValue}
            onChange={(event) => onSelectMonth(event.target.value)}
          />
          <button
            className="flex h-9 w-9 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-600 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50"
            type="button"
            title="Next month"
            disabled={!canGoNextMonth}
            onClick={onNextMonth}
          >
            <ChevronRight className="h-4 w-4" />
          </button>
        </div>
      </section>

      <section className="col-span-2 grid min-h-0 grid-cols-[380px_minmax(0,1fr)] overflow-y-auto bg-white">
        <div className="min-w-0 border-r border-stone-200">
          <div className="sticky top-0 z-30 flex h-[76px] items-center justify-between gap-3 border-b border-stone-200 bg-slate-800 px-4 text-white">
            <div className="min-w-0">
              <h3 className="truncate text-sm font-bold">Project / task</h3>
              <p className="mt-1 truncate text-xs text-white/70">Grouped by assigned project</p>
            </div>
            <span className="rounded-md bg-white/10 px-2 py-1 text-xs font-bold">Total</span>
          </div>

          {projects.map((project) => (
            <ProjectTaskRows
              key={project.id}
              entries={entries}
              project={project}
              totalHours={totalHours}
            />
          ))}
        </div>

        <div className="min-w-0 overflow-x-auto">
          <div className="min-w-max">
            <div className="sticky top-0 z-20 flex h-[76px] border-b border-stone-200 bg-slate-800 text-white">
              {days.map((day) => (
                <div
                  key={day.day}
                  className={[
                    "flex w-12 shrink-0 flex-col items-center justify-center border-r border-white/10 px-1 text-center",
                    day.isToday ? "bg-brand" : day.isWeekend ? "bg-slate-700" : "",
                  ].join(" ")}
                >
                  <span className="text-sm font-extrabold leading-none">{day.label}</span>
                  <span className="mt-1 text-[8px] font-semibold text-white/30">{day.weekday}</span>
                  <span className="mt-1 text-[11px] font-extrabold leading-none text-red-600">{formatHours(dayTotal(projects, entries, day.day))}h</span>
                </div>
              ))}
            </div>

            {projects.map((project) => (
            <ProjectDayRows
              key={project.id}
              days={days}
              entries={entries}
              onEditCell={setEditingCell}
              project={project}
            />
          ))}
          </div>
        </div>
      </section>

      {editingCell ? (
        <DailyReportEntryDialog
          cell={editingCell}
          onClose={() => setEditingCell(null)}
          onDelete={() => {
            onUpdateEntry(editingCell.task.id, editingCell.day.day, null);
            setEditingCell(null);
          }}
          onSave={(entry) => {
            onUpdateEntry(editingCell.task.id, editingCell.day.day, entry);
            setEditingCell(null);
          }}
        />
      ) : null}
    </section>
  );
}

function ProjectTaskRows({
  entries,
  project,
  totalHours,
}: {
  entries: DailyReportEntries;
  project: DailyReportProject;
  totalHours: (taskId?: string) => number;
}) {
  return (
    <>
      <div className={projectRowClass}>
        <div className="min-w-0 flex-1">
          <strong className="block truncate text-sm text-white">
            {project.code} - {project.name}
          </strong>
        </div>
        <span className="ml-3 shrink-0 rounded-md bg-white/20 px-2 py-1 text-xs font-bold text-white">
          {formatHours(projectTotal(project, entries))}h
        </span>
      </div>
      {project.tasks.map((task) => (
        <TaskRow key={task.id} task={task} total={totalHours(task.id)} />
      ))}
    </>
  );
}

function ProjectDayRows({
  days,
  entries,
  onEditCell,
  project,
}: {
  days: DailyReportDay[];
  entries: DailyReportEntries;
  onEditCell: (cell: EditingCell) => void;
  project: DailyReportProject;
}) {
  return (
    <>
      <div className="flex h-12 border-b border-emerald-200 bg-[#4cbd9b]">
        {days.map((day) => (
          <div
            key={`${project.id}-${day.day}`}
            className={[
              "flex h-12 w-12 shrink-0 items-center justify-center border-r border-white/20 px-1 text-xs font-extrabold tabular-nums text-white",
              day.isWeekend ? "bg-white/10" : "",
            ].join(" ")}
            title={`${project.code} total - ${day.label} ${day.weekday}`}
          >
            {formatHours(projectDayTotal(project, entries, day.day))}
          </div>
        ))}
      </div>
      {project.tasks.map((task) => (
        <div key={task.id} className="flex h-14 border-b border-stone-200">
          {days.map((day) => (
            <DailyHourCell
              key={`${task.id}-${day.day}`}
              day={day}
              entry={entries[entryKey(task.id, day.day)]}
              task={task}
              onEdit={onEditCell}
            />
          ))}
        </div>
      ))}
    </>
  );
}

function TaskRow({ task, total }: { task: DailyReportTask; total: number }) {
  return (
    <div className={taskRowClass}>
      <div className="min-w-0 flex-1 pl-3">
        <strong className="block truncate text-sm text-slate-900">{task.name}</strong>
        <span className="mt-1 block truncate text-xs font-semibold text-slate-500">{task.code}</span>
      </div>
      <span className="ml-3 shrink-0 rounded-md bg-slate-100 px-2 py-1 text-xs font-bold text-slate-600">
        {formatHours(total)}h
      </span>
    </div>
  );
}

function DailyHourCell({
  day,
  entry,
  task,
  onEdit,
}: {
  day: DailyReportDay;
  entry: DailyReportEntry | string | undefined;
  task: DailyReportTask;
  onEdit: (cell: EditingCell) => void;
}) {
  const hour = entryHour(entry);
  const hasValue = hour > 0;

  return (
    <div className={[dayCellClass, day.isWeekend ? "bg-slate-50" : "bg-white"].join(" ")}>
      <button
        className={[
          "flex h-9 w-10 items-center justify-center rounded-md border text-sm font-bold tabular-nums outline-none transition focus:ring-2 focus:ring-emerald-100",
          hasValue
            ? "border-brand bg-emerald-50 text-brand hover:bg-emerald-100"
            : "border-slate-200 bg-white text-slate-400 hover:border-brand hover:text-brand",
        ].join(" ")}
        type="button"
        title={`${task.name} - ${day.label} ${day.weekday}`}
        onClick={() => onEdit({ day, entry: normalizeEntryForForm(entry), task })}
      >
        {hasValue ? formatHours(hour) : <Plus className="h-4 w-4" />}
      </button>
    </div>
  );
}

function dayTotal(projects: DailyReportProject[], entries: DailyReportEntries, day: number) {
  return projects.reduce((total, project) => total + projectDayTotal(project, entries, day), 0);
}

function projectDayTotal(project: DailyReportProject, entries: DailyReportEntries, day: number) {
  return project.tasks.reduce((total, task) => total + entryHour(entries[entryKey(task.id, day)]), 0);
}

function projectTotal(project: DailyReportProject, entries: DailyReportEntries) {
  return project.tasks.reduce((total, task) => {
    return total + Object.entries(entries).reduce((taskTotal, [key, value]) => {
      return key.startsWith(`${task.id}:`) ? taskTotal + entryHour(value) : taskTotal;
    }, 0);
  }, 0);
}

function formatHours(value: number) {
  return Number.isInteger(value) ? value.toLocaleString("en-US") : value.toFixed(2);
}

type EditingCell = {
  day: DailyReportDay;
  entry: DailyReportEntry;
  task: DailyReportTask;
};

function DailyReportEntryDialog({
  cell,
  onClose,
  onDelete,
  onSave,
}: {
  cell: EditingCell;
  onClose: () => void;
  onDelete: () => void;
  onSave: (entry: DailyReportEntry) => void;
}) {
  const [form, setForm] = useState<DailyReportEntry>(cell.entry);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/40 p-4">
      <section className="w-full max-w-md rounded-lg bg-white shadow-xl">
        <div className="border-b border-stone-200 px-5 py-4">
          <h3 className="font-bold text-slate-900">Daily report detail</h3>
          <p className="mt-1 text-sm text-slate-500">
            {cell.task.name} - {cell.day.label} {cell.day.weekday}
          </p>
        </div>

        <div className="space-y-4 px-5 py-4">
          <label className="block">
            <span className="text-xs font-bold text-slate-500">Hour</span>
            <input
              className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              inputMode="decimal"
              max="24"
              min="0"
              step="0.25"
              type="number"
              value={form.hour}
              onChange={(event) => setForm((current) => ({ ...current, hour: event.target.value }))}
            />
          </label>

          <label className="block">
            <span className="text-xs font-bold text-slate-500">Phase</span>
            <input
              className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              value={form.phase}
              onChange={(event) => setForm((current) => ({ ...current, phase: event.target.value }))}
            />
          </label>

          <label className="block">
            <span className="text-xs font-bold text-slate-500">Comment</span>
            <textarea
              className="mt-1 min-h-24 w-full resize-none rounded-md border border-slate-300 bg-white px-3 py-2 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              value={form.comment}
              onChange={(event) => setForm((current) => ({ ...current, comment: event.target.value }))}
            />
          </label>

          <label className="flex items-center gap-2 text-sm font-bold text-slate-700">
            <input
              className="h-4 w-4 accent-brand"
              checked={form.isOt}
              type="checkbox"
              onChange={(event) =>
                setForm((current) => ({
                  ...current,
                  isOt: event.target.checked,
                  midnightOt: event.target.checked ? current.midnightOt : "",
                  regularOt: event.target.checked ? current.regularOt : "",
                }))
              }
            />
            OT
          </label>

          {form.isOt ? (
            <div className="grid gap-2 rounded-md border border-slate-200 bg-slate-50 p-3">
              <label className="block">
                <span className="text-xs font-bold text-slate-500">Regular OT</span>
                <input
                  className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  inputMode="decimal"
                  max="24"
                  min="0"
                  step="0.25"
                  type="number"
                  value={form.regularOt}
                  onChange={(event) => setForm((current) => ({ ...current, regularOt: event.target.value }))}
                />
              </label>
              <label className="block">
                <span className="text-xs font-bold text-slate-500">Midnight OT</span>
                <input
                  className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                  inputMode="decimal"
                  max="24"
                  min="0"
                  step="0.25"
                  type="number"
                  value={form.midnightOt}
                  onChange={(event) => setForm((current) => ({ ...current, midnightOt: event.target.value }))}
                />
              </label>
            </div>
          ) : null}
        </div>

        <div className="flex items-center justify-between gap-3 border-t border-stone-200 px-5 py-4">
          <button
            className="h-10 rounded-md border border-red-200 bg-white px-4 text-sm font-bold text-red-600 hover:bg-red-50"
            type="button"
            onClick={onDelete}
          >
            Clear
          </button>
          <div className="flex items-center gap-2">
            <button
              className="h-10 rounded-md border border-slate-300 bg-white px-4 text-sm font-bold text-slate-600 hover:bg-slate-50"
              type="button"
              onClick={onClose}
            >
              Cancel
            </button>
            <button
              className="h-10 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90"
              type="button"
              onClick={() => onSave(form)}
            >
              Save
            </button>
          </div>
        </div>
      </section>
    </div>
  );
}

function normalizeEntryForForm(entry: DailyReportEntry | string | undefined): DailyReportEntry {
  if (!entry) {
    return emptyEntry();
  }

  if (typeof entry === "string") {
    return {
      ...emptyEntry(),
      hour: entry,
    };
  }

  return entry;
}

function emptyEntry(): DailyReportEntry {
  return {
    comment: "",
    hour: "",
    isOt: false,
    midnightOt: "",
    phase: "",
    regularOt: "",
  };
}
