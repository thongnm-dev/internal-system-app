import { ChevronLeft, ChevronRight, Plus, RotateCcw, Search } from "lucide-react";
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
        <button
          className="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90"
          type="button"
          onClick={() => onNavigate("/projects/detail")}
        >
          <Plus className="h-4 w-4" />
          Register
        </button>
      </section>

      <fieldset className="rounded-lg border border-stone-200 bg-panel p-4 shadow-sm">
        <legend className="px-2 text-sm font-bold text-slate-600">Search conditions</legend>
        <div className="grid gap-3">
          <div className="grid gap-3 lg:grid-cols-2">
            <label>
              <span className="text-xs font-bold text-slate-500">Project Code</span>
              <input
                className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
                placeholder="Code"
                type="search"
                value={filters.code}
                onChange={(event) => onSetFilters((current) => ({ ...current, code: event.target.value }))}
              />
            </label>
            <label>
              <span className="text-xs font-bold text-slate-500">Project Name</span>
              <input
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
            <input
              className="mt-1 h-10 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
              placeholder="Code, name, phase, work content"
              type="search"
              value={filters.keyword}
              onChange={(event) => onSetFilters((current) => ({ ...current, keyword: event.target.value }))}
            />
          </label>
          <div className="flex items-center justify-end gap-2">
            <button
              className="flex h-10 items-center gap-2 rounded-md bg-brand px-4 text-sm font-bold text-white hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60"
              type="button"
              title="Search"
              disabled={isSearching}
              onClick={onSearch}
            >
              <Search className="h-4 w-4" />
              Search
            </button>
            <button
              className="flex h-10 items-center gap-2 rounded-md border border-slate-300 bg-white px-4 text-sm font-bold text-slate-600 hover:bg-slate-50"
              type="button"
              title="Reset search conditions"
              onClick={onReset}
            >
              <RotateCcw className="h-4 w-4" />
              Reset
            </button>
          </div>
        </div>
        {searchError ? <p className="mt-3 text-sm text-red-600">{searchError}</p> : null}
      </fieldset>

      <section className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-stone-200 bg-panel shadow-sm">
        <div className="flex items-center justify-between gap-4 border-b border-stone-200 px-4 py-3">
          <h3 className="font-bold">Project list</h3>
          <span className="text-xs text-slate-500">
            {filteredProjects.length.toLocaleString("en-US")} projects
            {result?.source_path ? ` from ${result.source_path}` : ""}
          </span>
        </div>
        <div className="min-h-0 overflow-auto">
          <table className="w-full min-w-[980px] border-collapse">
            <thead>
              <tr>
                <th className="table-head">Code</th>
                <th className="table-head">Name</th>
                <th className="table-head num">Total hour</th>
                <th className="table-head num">Bug count</th>
              </tr>
            </thead>
            <tbody>
              {!result || result.projects.length === 0 ? (
                <tr>
                  <td className="table-cell h-40 text-center text-slate-500" colSpan={4}>
                    No analysis data yet.
                  </td>
                </tr>
              ) : visibleProjects.length === 0 ? (
                <tr>
                  <td className="table-cell h-40 text-center text-slate-500" colSpan={4}>
                    No projects match the search conditions.
                  </td>
                </tr>
              ) : (
                visibleProjects.map((project) => (
                  <tr
                    key={project.project_code}
                    className="cursor-pointer hover:bg-slate-50"
                    title="Open project detail"
                    onClick={() => onNavigate(`/projects/detail/${encodeURIComponent(project.project_code)}`)}
                  >
                    <td className="table-cell font-bold text-ink">{project.project_code || "-"}</td>
                    <td className="table-cell">{project.project_name || "-"}</td>
                    <td className="table-cell num font-extrabold text-brand">{formatHourValue(totalMinutes(project.totals))}</td>
                    <td className="table-cell num">{bugCount(project).toLocaleString("en-US")}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
        <div className="flex items-center justify-between gap-4 border-t border-stone-200 px-4 py-3">
          <span className="text-sm text-slate-500">
            Page {page.toLocaleString("en-US")} / {pageCount.toLocaleString("en-US")}
          </span>
          <div className="flex items-center gap-2">
            <button
              className="flex h-9 w-9 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-600 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={page <= 1}
              type="button"
              title="Previous page"
              onClick={() => setPage((current) => Math.max(1, current - 1))}
            >
              <ChevronLeft className="h-4 w-4" />
            </button>
            <button
              className="flex h-9 w-9 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-600 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={page >= pageCount}
              type="button"
              title="Next page"
              onClick={() => setPage((current) => Math.min(pageCount, current + 1))}
            >
              <ChevronRight className="h-4 w-4" />
            </button>
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
