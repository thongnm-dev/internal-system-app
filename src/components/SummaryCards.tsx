import type { SummaryMetric } from "../types/statistics";

type SummaryCardsProps = {
  metrics: SummaryMetric[];
};

export function SummaryCards({ metrics }: SummaryCardsProps) {
  return (
    <div className="grid grid-cols-4 gap-3">
      {metrics.map((metric) => (
        <div key={metric.label} className="rounded-lg border border-stone-200 bg-white p-4 shadow-sm">
          <span className="text-sm font-bold text-slate-500">{metric.label}</span>
          <strong className="mt-2 block text-2xl leading-tight text-slate-900">{metric.value}</strong>
        </div>
      ))}
    </div>
  );
}
