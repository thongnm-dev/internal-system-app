import { FolderOpen, RefreshCw } from "lucide-react";

type HeaderProps = {
  csvPath: string;
  isLoading: boolean;
  onAnalyze: () => void;
  onCsvPathChange: (value: string) => void;
  onPickCsvFile: () => void;
};

export function Header({ csvPath, isLoading, onAnalyze, onCsvPathChange, onPickCsvFile }: HeaderProps) {
  return (
    <header className="grid grid-cols-[minmax(0,1fr)_minmax(430px,520px)] items-end gap-6">
      <div>
        <h2 className="text-2xl font-bold leading-tight">Detail Dashboard</h2>
        <p className="mt-2 text-sm text-slate-600">
          Group by <strong>Project Code</strong> and development phase from <strong>Process Code</strong>.
        </p>
      </div>

      <form
        className="grid gap-2"
        onSubmit={(event) => {
          event.preventDefault();
          onAnalyze();
        }}
      >
        <label className="text-xs font-bold uppercase tracking-wide text-slate-500" htmlFor="csv-path">
          CSV path
        </label>
        <div className="grid grid-cols-[minmax(0,1fr)_42px_42px] gap-2">
          <input
            id="csv-path"
            className="h-10 min-w-0 rounded-md border border-slate-300 bg-white px-3 text-sm outline-none focus:border-brand focus:ring-2 focus:ring-emerald-100"
            type="text"
            value={csvPath}
            onChange={(event) => onCsvPathChange(event.target.value)}
          />
          <button
            className="flex h-10 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-700 hover:bg-slate-50"
            type="button"
            title="Browse CSV"
            onClick={onPickCsvFile}
          >
            <FolderOpen className="h-4 w-4" />
          </button>
          <button
            className="flex h-10 items-center justify-center rounded-md bg-brand text-white hover:bg-emerald-800 disabled:cursor-not-allowed disabled:opacity-60"
            type="submit"
            title="Analyze"
            disabled={isLoading}
          >
            <RefreshCw className={["h-4 w-4", isLoading ? "animate-spin" : ""].join(" ")} />
          </button>
        </div>
      </form>
    </header>
  );
}
