import { ref } from "vue";
import type { AnalysisResult, MinuteTotals } from "@/shared/types/analysis";

export type ProjectFilters = {
  code: string;
  keyword: string;
  name: string;
};

const zeroTotals: MinuteTotals = { regular_minutes: 0, normal_overtime_minutes: 0, legal_holiday_overtime_minutes: 0, legal_public_holiday_overtime_minutes: 0, late_night_overtime_minutes: 0 };

const mockProjectResult: AnalysisResult = {
  source_path: "mock/projects.csv",
  row_count: 18,
  grand_total: { regular_minutes: 10680, normal_overtime_minutes: 960, legal_holiday_overtime_minutes: 0, legal_public_holiday_overtime_minutes: 0, late_night_overtime_minutes: 180 },
  projects: [
    {
      project_code: "INT-SYS", project_name: "Internal System App", row_count: 6,
      totals: { regular_minutes: 2880, normal_overtime_minutes: 300, legal_holiday_overtime_minutes: 0, legal_public_holiday_overtime_minutes: 0, late_night_overtime_minutes: 60 },
      phases: [
        { process_code: "ARCH", phase_name: "Architecture", row_count: 2, totals: { ...zeroTotals, regular_minutes: 960 }, details: [{ date: "2026-05-11", work_content: "Define Tauri command architecture", total_minutes: 480 }, { date: "2026-05-12", work_content: "Review frontend routing structure", total_minutes: 480 }] },
        { process_code: "DEV", phase_name: "Development", row_count: 4, totals: { ...zeroTotals, regular_minutes: 1920, normal_overtime_minutes: 300, late_night_overtime_minutes: 60 }, details: [{ date: "2026-05-13", work_content: "Implement project SKILL.md management", total_minutes: 540 }, { date: "2026-05-14", work_content: "Fix UI bug in focus border", total_minutes: 420 }, { date: "2026-05-15", work_content: "Add project context menu", total_minutes: 600 }, { date: "2026-05-16", work_content: "Review generated SKILL.md preview", total_minutes: 720 }] },
      ],
    },
    {
      project_code: "REP-HUB", project_name: "Reporting Hub", row_count: 5,
      totals: { regular_minutes: 3300, normal_overtime_minutes: 180, legal_holiday_overtime_minutes: 0, legal_public_holiday_overtime_minutes: 0, late_night_overtime_minutes: 0 },
      phases: [
        { process_code: "BE", phase_name: "Backend", row_count: 3, totals: { ...zeroTotals, regular_minutes: 1980 }, details: [{ date: "2026-05-06", work_content: "Build report aggregation API", total_minutes: 720 }, { date: "2026-05-07", work_content: "Optimize PostgreSQL monthly summary query", total_minutes: 660 }, { date: "2026-05-08", work_content: "Fix defect in timezone conversion", total_minutes: 600 }] },
        { process_code: "FE", phase_name: "Frontend", row_count: 2, totals: { ...zeroTotals, regular_minutes: 1320, normal_overtime_minutes: 180 }, details: [{ date: "2026-05-09", work_content: "Create dashboard filter panel", total_minutes: 720 }, { date: "2026-05-10", work_content: "Review chart interaction behavior", total_minutes: 780 }] },
      ],
    },
    {
      project_code: "BIL-PORTAL", project_name: "Billing Portal", row_count: 4,
      totals: { regular_minutes: 2460, normal_overtime_minutes: 300, legal_holiday_overtime_minutes: 0, legal_public_holiday_overtime_minutes: 0, late_night_overtime_minutes: 120 },
      phases: [
        { process_code: "PAY", phase_name: "Payment", row_count: 4, totals: { ...zeroTotals, regular_minutes: 2460, normal_overtime_minutes: 300, late_night_overtime_minutes: 120 }, details: [{ date: "2026-05-02", work_content: "Implement reconciliation export", total_minutes: 780 }, { date: "2026-05-03", work_content: "Investigate payment issue retry flow", total_minutes: 600 }, { date: "2026-05-04", work_content: "Fix bug in invoice rounding", total_minutes: 660 }, { date: "2026-05-05", work_content: "Review settlement edge cases", total_minutes: 840 }] },
      ],
    },
    {
      project_code: "MOB-GW", project_name: "Mobile Gateway", row_count: 3,
      totals: { regular_minutes: 2040, normal_overtime_minutes: 180, legal_holiday_overtime_minutes: 0, legal_public_holiday_overtime_minutes: 0, late_night_overtime_minutes: 0 },
      phases: [
        { process_code: "AUTH", phase_name: "Authentication", row_count: 3, totals: { ...zeroTotals, regular_minutes: 2040, normal_overtime_minutes: 180 }, details: [{ date: "2026-05-13", work_content: "Refresh token implementation", total_minutes: 780 }, { date: "2026-05-14", work_content: "Fix security issue in session renewal", total_minutes: 660 }, { date: "2026-05-15", work_content: "Review mobile network switching behavior", total_minutes: 780 }] },
      ],
    },
  ],
};

export function useProjects() {
  const filters = ref<ProjectFilters>({ code: "", keyword: "", name: "" });
  const result = ref<AnalysisResult | null>(mockProjectResult);

  function searchProjects() {}
  function resetFilters() { filters.value = { code: "", keyword: "", name: "" }; }

  return { filters, isSearching: false, result, resetFilters, searchError: "", searchProjects, setFilters: (f: ProjectFilters) => { filters.value = f; } };
}
