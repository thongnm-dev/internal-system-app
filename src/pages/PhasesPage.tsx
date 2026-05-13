import { Clock3 } from "lucide-react";
import { useMemo } from "react";
import { addTotals, emptyTotals, formatHours, totalMinutes } from "../core/timeMath";
import type { AnalysisResult, MinuteTotals } from "../types/statistics";

type PhasesPageProps = {
  result: AnalysisResult | null;
};

type PhaseTotal = {
  code: string;
  name: string;
  totals: MinuteTotals;
  rows: number;
};

export function PhasesPage({ result }: PhasesPageProps) {
  const phaseTotals = useMemo(() => {
    const map = new Map<string, PhaseTotal>();
    for (const project of result?.projects ?? []) {
      for (const phase of project.phases) {
        const current =
          map.get(phase.process_code) ??
          ({
            code: phase.process_code,
            name: phase.phase_name,
            totals: emptyTotals(),
            rows: 0,
          } satisfies PhaseTotal);

        current.rows += phase.row_count;
        addTotals(current.totals, phase.totals);
        map.set(phase.process_code, current);
      }
    }
    return Array.from(map.values()).sort((a, b) => totalMinutes(b.totals) - totalMinutes(a.totals));
  }, [result]);

  return (
    <section className="min-h-0 flex-1 overflow-auto rounded-lg border border-stone-200 bg-white p-4 shadow-sm">
      <div className="flex items-center gap-2">
        <Clock3 className="h-5 w-5 text-brand" />
        <h3 className="font-bold">Phase summary</h3>
      </div>
      <div className="mt-4 grid grid-cols-2 gap-3">
        {phaseTotals.length === 0 ? (
          <p className="text-sm text-slate-500">No data yet.</p>
        ) : (
          phaseTotals.map((phase) => (
            <div key={phase.code} className="rounded-lg border border-stone-200 p-4">
              <div className="flex items-center justify-between gap-4">
                <div>
                  <span className="inline-block rounded bg-blue-100 px-2 py-1 text-xs font-extrabold text-blue-800">
                    {phase.code}
                  </span>
                  <h4 className="mt-2 font-bold">{phase.name}</h4>
                </div>
                <strong className="text-xl text-brand">{formatHours(totalMinutes(phase.totals))}</strong>
              </div>
              <p className="mt-2 text-sm text-slate-500">{phase.rows.toLocaleString("en-US")} rows</p>
            </div>
          ))
        )}
      </div>
    </section>
  );
}
