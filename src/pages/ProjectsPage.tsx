import { ChevronLeft, ChevronRight, Plus, RotateCcw, Search } from "lucide-react";
import { Button } from "primereact/button";
import { Column } from "primereact/column";
import { DataTable } from "primereact/datatable";
import { Fieldset } from "primereact/fieldset";
import { InputText } from "primereact/inputtext";
import { useEffect, useMemo, useState } from "react";
import type { Dispatch, SetStateAction } from "react";
import type { ProjectFilters } from "../controller/useProjectsController";
import { formatHourValue, totalMinutes } from "../core/timeMath";
import type { AnalysisResult, ProjectSummary } from "../types/statistics";

type ProjectsPageProps = {
  filters: ProjectFilters;
  isSearching: boolean;
  onNavigate: (path: string) => void;
  onReset: () => void;
  onSearch: () => void;
  onSetFilters: Dispatch<SetStateAction<ProjectFilters>>;
  result: AnalysisResult | null;
  searchError: string;
};

const pageSize = 10;

export function ProjectsPage({
  filters,
  isSearching,
  result,
  searchError,
  onNavigate,
  onReset,
  onSearch,
  onSetFilters,
}: ProjectsPageProps) {
  const [page, setPage] = useState(1);

  const filteredProjects = useMemo(() => {
    const projects = result?.projects ?? [];
    const code = normalize(filters.code);
    const name = normalize(filters.name);
    const keyword = normalize(filters.keyword);

    return projects.filter((project) => {
      const matchesCode = !code || normalize(project.project_code).includes(code);
      const matchesName = !name || normalize(project.project_name).includes(name);
      const matchesKeyword =
        !keyword ||
        normalize(
          [
            project.project_code,
            project.project_name,
            ...project.phases.flatMap((phase) => [
              phase.process_code,
              phase.phase_name,
              ...phase.details.map((detail) => detail.work_content),
            ]),
          ].join(" "),
        ).includes(keyword);

      return matchesCode && matchesName && matchesKeyword;
    });
  }, [filters, result]);

  const pageCount = Math.max(1, Math.ceil(filteredProjects.length / pageSize));
  const visibleProjects = filteredProjects.slice((page - 1) * pageSize, page * pageSize);

  useEffect(() => {
    setPage(1);
  }, [filters.code, filters.keyword, filters.name]);

  useEffect(() => {
    setPage((current) => Math.min(current, pageCount));
  }, [pageCount]);

  return (
    <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
      <section className="flex items-center justify-end rounded-lg border border-stone-200 bg-panel p-4 shadow-sm">
        <Button
          className="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90"
          type="button"
          onClick={() => onNavigate("/projects/detail")}
        >
          <Plus className="h-4 w-4" />
          Register
        </Button>
      </section>

      <Fieldset className="rounded-lg border border-slate-200 bg-white p-4 shadow-md fieldset-nested" legend="Search" toggleable>
        <div className="grid gap-3">
          <div className="grid gap-3 lg:grid-cols-2">
            <label>
              <span className="text-xs font-bold text-slate-500">Project Code</span>
              <InputText
                className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                placeholder="Code"
                type="search"
                value={filters.code}
                onChange={(event) => onSetFilters((current) => ({ ...current, code: event.target.value }))}
              />
            </label>
            <label>
              <span className="text-xs font-bold text-slate-500">Project Name</span>
              <InputText
                className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                placeholder="Name"
                type="search"
                value={filters.name}
                onChange={(event) => onSetFilters((current) => ({ ...current, name: event.target.value }))}
              />
            </label>
          </div>
          <label>
            <span className="text-xs font-bold text-slate-500">Keyword Search</span>
            <InputText
              className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Code, name, phase, work content"
              type="search"
              value={filters.keyword}
              onChange={(event) => onSetFilters((current) => ({ ...current, keyword: event.target.value }))}
            />
          </label>
          <div className="flex items-center justify-end gap-2">
            <Button
              className="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60"
              type="button"
              title="Search"
              disabled={isSearching}
              onClick={onSearch}
            >
              <Search className="h-4 w-4" />
              Search
            </Button>
            <Button
              className="flex h-10 items-center gap-2 rounded-md border border-slate-300 bg-white px-4 text-sm font-bold text-slate-600 hover:bg-slate-50"
              type="button"
              title="Reset search conditions"
              onClick={onReset}
            >
              <RotateCcw className="h-4 w-4" />
              Reset
            </Button>
          </div>
        </div>
        {searchError ? <p className="mt-3 text-sm text-red-600">{searchError}</p> : null}
      </Fieldset>

      <section className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-stone-200 bg-panel shadow-sm">
        <div className="flex items-center justify-between gap-4 border-b border-stone-200 px-4 py-3">
          <h3 className="font-bold">Project list</h3>
          <span className="text-xs text-slate-500">
            {filteredProjects.length.toLocaleString("en-US")} projects
            {result?.source_path ? ` from ${result.source_path}` : ""}
          </span>
        </div>
        <DataTable
          className="app-data-table min-h-0"
          emptyMessage={!result || result.projects.length === 0 ? "No analysis data yet." : "No projects match the search conditions."}
          rowClassName={() => "cursor-pointer"}
          scrollable
          scrollHeight="flex"
          tableStyle={{ minWidth: "980px" }}
          value={visibleProjects}
          onRowClick={(event) => onNavigate(`/projects/detail/${encodeURIComponent(event.data.project_code)}`)}
        >
          <Column field="project_code" header="Code" body={(project: ProjectSummary) => project.project_code || "-"} bodyClassName="font-bold text-ink" />
          <Column field="project_name" header="Name" body={(project: ProjectSummary) => project.project_name || "-"} />
          <Column
            header="Total hour"
            body={(project: ProjectSummary) => formatHourValue(totalMinutes(project.totals))}
            bodyClassName="num font-extrabold text-brand"
            headerClassName="num"
          />
          <Column
            header="Bug count"
            body={(project: ProjectSummary) => bugCount(project).toLocaleString("en-US")}
            bodyClassName="num"
            headerClassName="num"
          />
        </DataTable>
        <div className="flex items-center justify-between gap-4 border-t border-stone-200 px-4 py-3">
          <span className="text-sm text-slate-500">
            Page {page.toLocaleString("en-US")} / {pageCount.toLocaleString("en-US")}
          </span>
          <div className="flex items-center gap-2">
            <Button
              className="flex h-9 w-9 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-600 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={page <= 1}
              type="button"
              title="Previous page"
              onClick={() => setPage((current) => Math.max(1, current - 1))}
            >
              <ChevronLeft className="h-4 w-4" />
            </Button>
            <Button
              className="flex h-9 w-9 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-600 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={page >= pageCount}
              type="button"
              title="Next page"
              onClick={() => setPage((current) => Math.min(pageCount, current + 1))}
            >
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </section>
    </section>
  );
}

function bugCount(project: ProjectSummary) {
  return project.phases.reduce(
    (total, phase) =>
      total +
      phase.details.filter((detail) => /\b(bug|defect|issue)\b/i.test(detail.work_content)).length,
    0,
  );
}

function normalize(value: string) {
  return value.trim().toLocaleLowerCase();
}
