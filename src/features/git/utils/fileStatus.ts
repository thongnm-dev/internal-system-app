const STATUS_META: Record<string, { label: string; cls: string; badge: string }> = {
  M: { label: "Modified", cls: "text-amber-600", badge: "bg-amber-100 text-amber-700" },
  A: { label: "Added", cls: "text-emerald-600", badge: "bg-emerald-100 text-emerald-700" },
  D: { label: "Deleted", cls: "text-red-600", badge: "bg-red-100 text-red-700" },
  R: { label: "Renamed", cls: "text-sky-600", badge: "bg-sky-100 text-sky-700" },
  C: { label: "Copied", cls: "text-sky-600", badge: "bg-sky-100 text-sky-700" },
  U: { label: "Conflict", cls: "text-red-600", badge: "bg-red-100 text-red-700" },
  "?": { label: "New", cls: "text-emerald-600", badge: "bg-emerald-100 text-emerald-700" },
};

export function statusMeta(code: string) {
  return STATUS_META[code] ?? { label: code, cls: "text-muted", badge: "bg-slate-100 text-slate-600" };
}

export function baseName(path: string) {
  const parts = path.split("/");
  return parts[parts.length - 1] || path;
}

export function dirName(path: string) {
  const idx = path.lastIndexOf("/");
  return idx > 0 ? path.slice(0, idx) : "";
}
