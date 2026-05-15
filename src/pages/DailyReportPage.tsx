import { CalendarDays, ChevronLeft, ChevronRight, FolderOpen, Plus, Trash2, Upload } from "lucide-react";
import { Button } from "primereact/button";
import { Calendar } from "primereact/calendar";
import { Checkbox } from "primereact/checkbox";
import { Dialog } from "primereact/dialog";
import { InputText } from "primereact/inputtext";
import { InputTextarea } from "primereact/inputtextarea";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import type { MouseEvent as ReactMouseEvent, ReactNode, UIEvent, WheelEvent } from "react";
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
  onRemoveProject: (projectId: string) => void;
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
  onRemoveProject,
  onSelectMonth,
  onUpdateEntry,
  projects,
  totalHours,
}: DailyReportPageProps) {
  const [isAddingProject, setIsAddingProject] = useState(false);
  const [editingCell, setEditingCell] = useState<EditingCell | null>(null);
  const dayHeaderRef = useRef<HTMLDivElement>(null);
  const dayRowsRef = useRef<HTMLDivElement>(null);
  const [dayRowsScrollbarHeight, setDayRowsScrollbarHeight] = useState(0);
  const [tableScrollTop, setTableScrollTop] = useState(0);
  const [projectMenu, setProjectMenu] = useState<ProjectContextMenuState | null>(null);

  useEffect(() => {
    if (!projectMenu) {
      return;
    }

    const closeMenu = () => setProjectMenu(null);
    window.addEventListener("click", closeMenu);
    window.addEventListener("contextmenu", closeMenu);
    window.addEventListener("keydown", closeMenu);

    return () => {
      window.removeEventListener("click", closeMenu);
      window.removeEventListener("contextmenu", closeMenu);
      window.removeEventListener("keydown", closeMenu);
    };
  }, [projectMenu]);

  useLayoutEffect(() => {
    const dayRows = dayRowsRef.current;

    if (!dayRows) {
      return;
    }

    const updateScrollbarHeight = () => {
      setDayRowsScrollbarHeight(dayRows.offsetHeight - dayRows.clientHeight);
    };

    updateScrollbarHeight();

    const resizeObserver = new ResizeObserver(updateScrollbarHeight);
    resizeObserver.observe(dayRows);

    return () => resizeObserver.disconnect();
  }, [days.length, projects.length]);

  const syncFromDayRows = (event: UIEvent<HTMLDivElement>) => {
    const scrollContainer = event.currentTarget;
    setTableScrollTop(scrollContainer.scrollTop);

    if (dayHeaderRef.current) {
      dayHeaderRef.current.scrollLeft = scrollContainer.scrollLeft;
    }
  };

  const scrollTableFromProjectColumn = (event: WheelEvent<HTMLDivElement>) => {
    if (!dayRowsRef.current) {
      return;
    }

    event.preventDefault();
    dayRowsRef.current.scrollTop += event.deltaY;
    dayRowsRef.current.scrollLeft += event.deltaX;
  };

  const openProjectMenu = (project: DailyReportProject, event: ReactMouseEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.stopPropagation();
    setProjectMenu({
      canDelete: Boolean(project.isUserAdded) && projectTotal(project, entries) === 0,
      project,
      x: event.clientX,
      y: event.clientY,
    });
  };

  const closeProjectMenu = () => setProjectMenu(null);

  const handleProjectMenuAction = (action: ProjectContextMenuAction, project: DailyReportProject) => {
    closeProjectMenu();

    if (action === "delete") {
      onRemoveProject(project.id);
    }
  };

  return (
    <section className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-stone-200 bg-panel shadow-sm">
      <section className="grid h-[76px] shrink-0 grid-cols-[380px_minmax(0,1fr)] border-b border-stone-200 bg-white">
        <div className="flex min-w-0 items-center justify-between gap-3 border-r border-stone-200 px-4">
          <div className="min-w-0">
            <h3 className="truncate font-bold">Assigned work</h3>
            <p className="mt-1 truncate text-xs text-slate-500">{projects.length.toLocaleString("en-US")} projects</p>
          </div>
          <Button
            className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-brand text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
            type="button"
            title="Add project"
            disabled={availableProjects.length === 0}
            onClick={() => setIsAddingProject(true)}
          >
            <Plus className="h-4 w-4" />
          </Button>
        </div>

        <div className="flex min-w-0 items-center justify-between gap-4 px-4">
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
            <Button
              className="flex h-9 w-9 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-600 hover:bg-slate-50"
              type="button"
              title="Previous month"
              onClick={onPreviousMonth}
            >
              <ChevronLeft className="h-4 w-4" />
            </Button>
            <Calendar
              className="h-9 rounded-md border border-slate-300 bg-white text-sm font-bold text-slate-700 outline-none focus-within:border-brand focus-within:ring-2 focus-within:ring-emerald-100"
              dateFormat="yy-mm"
              maxDate={parseMonth(maxMonthValue) ?? undefined}
              value={parseMonth(monthValue)}
              view="month"
              onChange={(event) => onSelectMonth(formatMonth(event.value instanceof Date ? event.value : null))}
            />
            <Button
              className="flex h-9 w-9 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-600 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50"
              type="button"
              title="Next month"
              disabled={!canGoNextMonth}
              onClick={onNextMonth}
            >
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </section>

      <section className="flex min-h-0 flex-1 flex-col overflow-hidden bg-white">
        <div className="grid h-[76px] shrink-0 grid-cols-[380px_minmax(0,1fr)] bg-slate-800 text-white">
          <div className="flex min-w-0 items-center justify-between gap-3 border-b border-r border-stone-200 px-4">
            <div className="min-w-0">
              <h3 className="truncate text-sm font-bold">Project / task</h3>
              <p className="mt-1 truncate text-xs text-white/70">Grouped by assigned project</p>
            </div>
            <span className="rounded-md bg-white/10 px-2 py-1 text-xs font-bold">Total</span>
          </div>

          <div ref={dayHeaderRef} className="min-w-0 overflow-hidden border-b border-stone-200">
            <div className="flex h-[76px] min-w-max">
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
          </div>
        </div>

        <div className="grid min-h-0 flex-1 grid-cols-[380px_minmax(0,1fr)]">
          <div className="min-w-0 overflow-hidden border-r border-stone-200" onWheel={scrollTableFromProjectColumn}>
            <div
              style={{
                transform: `translateY(-${tableScrollTop}px)`,
                willChange: "transform",
              }}
            >
              {projects.map((project) => (
                <ProjectTaskRows
                  key={project.id}
                  entries={entries}
                  project={project}
                  totalHours={totalHours}
                  onContextMenu={openProjectMenu}
                />
              ))}
              <div aria-hidden="true" style={{ height: dayRowsScrollbarHeight }} />
            </div>
          </div>

          <div ref={dayRowsRef} className="min-w-0 overflow-auto" onScroll={syncFromDayRows}>
            <div className="min-w-max">
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

      {isAddingProject ? (
        <ProjectPickerDialog
          projects={availableProjects}
          onClose={() => setIsAddingProject(false)}
          onSelect={(projectIds) => {
            projectIds.forEach(onAddProject);
            setIsAddingProject(false);
          }}
        />
      ) : null}

      {projectMenu ? (
        <ProjectContextMenu
          canDelete={projectMenu.canDelete}
          project={projectMenu.project}
          x={projectMenu.x}
          y={projectMenu.y}
          onAction={handleProjectMenuAction}
        />
      ) : null}
    </section>
  );
}

function ProjectPickerDialog({
  onClose,
  onSelect,
  projects,
}: {
  onClose: () => void;
  onSelect: (projectIds: string[]) => void;
  projects: DailyReportProject[];
}) {
  const [keyword, setKeyword] = useState("");
  const [selectedProjectIds, setSelectedProjectIds] = useState<string[]>([]);
  const normalizedKeyword = keyword.trim().toLowerCase();
  const filteredProjects = normalizedKeyword
    ? projects.filter((project) =>
        `${project.code} ${project.name}`.toLowerCase().includes(normalizedKeyword),
      )
    : projects;

  const toggleProject = (projectId: string) => {
    setSelectedProjectIds((current) =>
      current.includes(projectId) ? current.filter((id) => id !== projectId) : [...current, projectId],
    );
  };

  return (
    <Dialog
      className="w-full max-w-3xl overflow-hidden rounded-lg bg-white shadow-xl"
      contentClassName="p-0 min-h-0 flex flex-col"
      header={
          <div className="min-w-0">
            <h3 className="truncate font-bold text-slate-900">Add project</h3>
            <p className="mt-1 truncate text-sm text-slate-500">Select a project to add to daily report.</p>
          </div>
      }
      style={{ maxHeight: "86vh" }}
      visible
      onHide={onClose}
    >

        <div className="border-b border-stone-200 bg-slate-50 px-5 py-4">
          <label className="block">
            <span className="text-xs font-bold text-slate-500">Project code / name</span>
            <InputText
              className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Search by code or name"
              value={keyword}
              onChange={(event) => setKeyword(event.target.value)}
            />
          </label>
        </div>

        <div className="min-h-0 flex-1 overflow-auto p-3">
          {projects.length === 0 ? (
            <p className="px-2 py-6 text-center text-sm text-slate-500">No more projects.</p>
          ) : filteredProjects.length === 0 ? (
            <p className="px-2 py-6 text-center text-sm text-slate-500">No projects found.</p>
          ) : (
            <div className="grid gap-2">
              {filteredProjects.map((project) => {
                const isSelected = selectedProjectIds.includes(project.id);

                return (
                <label
                  key={project.id}
                  className={[
                    "flex cursor-pointer items-center gap-3 rounded-md border px-4 py-3 hover:border-brand hover:bg-emerald-50",
                    isSelected ? "border-brand bg-emerald-50" : "border-slate-200 bg-white",
                  ].join(" ")}
                >
                  <Checkbox
                    className="h-4 w-4 shrink-0 accent-brand"
                    checked={isSelected}
                    onChange={() => toggleProject(project.id)}
                  />
                  <span className="min-w-0">
                    <strong className="block truncate text-sm text-slate-900">
                      {project.code} - {project.name}
                    </strong>
                    <span className="mt-1 block truncate text-xs font-semibold text-slate-500">{project.client}</span>
                  </span>
                </label>
                );
              })}
            </div>
          )}
        </div>

        <div className="flex items-center justify-between gap-3 border-t border-stone-200 px-5 py-4">
          <span className="text-sm font-semibold text-slate-500">{selectedProjectIds.length} selected</span>
          <div className="flex items-center gap-2">
            <Button
              className="h-10 rounded-md border border-slate-300 bg-white px-4 text-sm font-bold text-slate-600 hover:bg-slate-50"
              type="button"
              onClick={onClose}
            >
              Cancel
            </Button>
            <Button
              className="h-10 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
              type="button"
              disabled={selectedProjectIds.length === 0}
              onClick={() => onSelect(selectedProjectIds)}
            >
              Add projects
            </Button>
          </div>
        </div>
    </Dialog>
  );
}

function ProjectTaskRows({
  entries,
  onContextMenu,
  project,
  totalHours,
}: {
  entries: DailyReportEntries;
  onContextMenu: (project: DailyReportProject, event: ReactMouseEvent<HTMLDivElement>) => void;
  project: DailyReportProject;
  totalHours: (taskId?: string) => number;
}) {
  return (
    <>
      <div className={projectRowClass} onContextMenu={(event) => onContextMenu(project, event)}>
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

type ProjectContextMenuAction = "addTask" | "backlog" | "importTask" | "delete";

type ProjectContextMenuState = {
  canDelete: boolean;
  project: DailyReportProject;
  x: number;
  y: number;
};

function ProjectContextMenu({
  canDelete,
  project,
  x,
  y,
  onAction,
}: {
  canDelete: boolean;
  project: DailyReportProject;
  x: number;
  y: number;
  onAction: (action: ProjectContextMenuAction, project: DailyReportProject) => void;
}) {
  return (
    <div
      className="fixed z-[60] w-52 overflow-hidden rounded-md border border-slate-200 bg-white py-1 text-sm text-slate-700 shadow-xl"
      style={{ left: x, top: y }}
      onClick={(event) => event.stopPropagation()}
      onContextMenu={(event) => event.preventDefault()}
    >
      <ContextMenuButton icon={<Plus className="h-4 w-4" />} label="Thêm task" onClick={() => onAction("addTask", project)} />
      <ContextMenuButton
        icon={<FolderOpen className="h-4 w-4" />}
        label="Xem backlog"
        onClick={() => onAction("backlog", project)}
      />
      <ContextMenuButton
        icon={<Upload className="h-4 w-4" />}
        label="Import task"
        onClick={() => onAction("importTask", project)}
      />
      {canDelete ? (
        <>
          <div className="my-1 border-t border-slate-100" />
          <ContextMenuButton
            danger
            icon={<Trash2 className="h-4 w-4" />}
            label="Xóa project"
            onClick={() => onAction("delete", project)}
          />
        </>
      ) : null}
    </div>
  );
}

function ContextMenuButton({
  danger,
  icon,
  label,
  onClick,
}: {
  danger?: boolean;
  icon: ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <Button
      className={[
        "flex w-full items-center gap-2 px-3 py-2 text-left font-semibold hover:bg-slate-50",
        danger ? "text-red-600 hover:bg-red-50" : "",
      ].join(" ")}
      type="button"
      onClick={onClick}
    >
      {icon}
      <span className="min-w-0 truncate">{label}</span>
    </Button>
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
      <Button
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
      </Button>
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

function parseMonth(value: string) {
  const [year, month] = value.split("-").map(Number);
  if (!year || !month) {
    return null;
  }

  return new Date(year, month - 1, 1);
}

function formatMonth(value: Date | null) {
  if (!value) {
    return "";
  }

  return `${value.getFullYear()}-${String(value.getMonth() + 1).padStart(2, "0")}`;
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
    <Dialog
      className="w-full max-w-md rounded-lg bg-white shadow-xl"
      contentClassName="p-0"
      header={
        <div>
          <h3 className="font-bold text-slate-900">Daily report detail</h3>
          <p className="mt-1 text-sm text-slate-500">
            {cell.task.name} - {cell.day.label} {cell.day.weekday}
          </p>
        </div>
      }
      visible
      onHide={onClose}
    >

        <div className="space-y-4 px-5 py-4">
          <label className="block">
            <span className="text-xs font-bold text-slate-500">Hour</span>
            <InputText
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
            <InputText
              className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              value={form.phase}
              onChange={(event) => setForm((current) => ({ ...current, phase: event.target.value }))}
            />
          </label>

          <label className="block">
            <span className="text-xs font-bold text-slate-500">Comment</span>
            <InputTextarea
              className="mt-1 min-h-24 w-full resize-none rounded-md border border-slate-300 bg-white px-3 py-2 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              value={form.comment}
              onChange={(event) => setForm((current) => ({ ...current, comment: event.target.value }))}
            />
          </label>

          <label className="flex items-center gap-2 text-sm font-bold text-slate-700">
            <Checkbox
              className="h-4 w-4 accent-brand"
              checked={form.isOt}
              onChange={(event) =>
                setForm((current) => ({
                  ...current,
                  isOt: Boolean(event.checked),
                  midnightOt: event.checked ? current.midnightOt : "",
                  regularOt: event.checked ? current.regularOt : "",
                }))
              }
            />
            OT
          </label>

          {form.isOt ? (
            <div className="grid gap-2 rounded-md border border-slate-200 bg-slate-50 p-3">
              <label className="block">
                <span className="text-xs font-bold text-slate-500">Regular OT</span>
                <InputText
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
                <InputText
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
          <Button
            className="h-10 rounded-md border border-red-200 bg-white px-4 text-sm font-bold text-red-600 hover:bg-red-50"
            type="button"
            onClick={onDelete}
          >
            Clear
          </Button>
          <div className="flex items-center gap-2">
            <Button
              className="h-10 rounded-md border border-slate-300 bg-white px-4 text-sm font-bold text-slate-600 hover:bg-slate-50"
              type="button"
              onClick={onClose}
            >
              Cancel
            </Button>
            <Button
              className="h-10 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90"
              type="button"
              onClick={() => onSave(form)}
            >
              Save
            </Button>
          </div>
        </div>
    </Dialog>
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
