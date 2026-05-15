import { LogOut } from "lucide-react";
import { Button } from "primereact/button";
import type { AppRoute } from "../router/routes";

type HeaderProps = {
  onLogout: () => void;
  route: AppRoute;
  username?: string;
};

export function Header({ onLogout, route, username }: HeaderProps) {
  return (
    <header className="flex items-start justify-between gap-4">
      <div>
        <h2 className="text-2xl font-bold leading-tight">{route.title}</h2>
        <p className="mt-2 text-sm text-slate-600">{route.subtitle}</p>
        <nav className="mt-3 flex items-center gap-2 text-xs font-semibold text-slate-500" aria-label="Breadcrumb">
          <span>Home</span>
          <span className="text-slate-300">/</span>
          <span className="text-brand">{route.title}</span>
        </nav>
      </div>
      {username && (
        <Button
          className="flex h-9 shrink-0 items-center justify-center gap-2 rounded-md border border-slate-300 bg-panel px-3 text-sm font-bold text-slate-700 hover:bg-slate-50"
          type="button"
          title="Logout"
          onClick={onLogout}
        >
          <LogOut className="h-4 w-4" />
          Logout
        </Button>
      )}
    </header>
  );
}
