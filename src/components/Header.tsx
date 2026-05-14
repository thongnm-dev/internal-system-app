import type { AppRoute } from "../router/routes";

type HeaderProps = {
  route: AppRoute;
};

export function Header({ route }: HeaderProps) {
  return (
    <header>
      <h2 className="text-2xl font-bold leading-tight">{route.title}</h2>
      <p className="mt-2 text-sm text-slate-600">{route.subtitle}</p>
      <nav className="mt-3 flex items-center gap-2 text-xs font-semibold text-slate-500" aria-label="Breadcrumb">
        <span>Home</span>
        <span className="text-slate-300">/</span>
        <span className="text-brand">{route.title}</span>
      </nav>
    </header>
  );
}
