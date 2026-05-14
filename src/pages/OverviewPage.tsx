import { Gauge } from "lucide-react";

type OverviewPageProps = {

};

export function OverviewPage({}: OverviewPageProps) {

  return (
    <section className="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_320px] gap-4 overflow-hidden">
      <section className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg border border-stone-200 bg-white shadow-sm">

      </section>
      <aside className="rounded-lg border border-stone-200 bg-white p-4 shadow-sm">
        <div className="flex items-center gap-2">
          <Gauge className="h-5 w-5 text-brand" />
          <h3 className="font-bold">Top projects</h3>
        </div>
        <div className="mt-4 space-y-3">
          
        </div>
      </aside>
    </section>
  );
}
