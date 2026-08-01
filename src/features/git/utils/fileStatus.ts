const STATUS_META: Record<string, { label: string; cls: string; badge: string }> = {
  M: { label: "Modified", cls: "text-warning", badge: "badge-warning" },
  A: { label: "Added", cls: "text-success", badge: "badge-success" },
  D: { label: "Deleted", cls: "text-danger", badge: "badge-danger" },
  R: { label: "Renamed", cls: "text-info", badge: "badge-info" },
  C: { label: "Copied", cls: "text-info", badge: "badge-info" },
  U: { label: "Conflict", cls: "text-danger", badge: "badge-danger" },
  "?": { label: "New", cls: "text-success", badge: "badge-success" },
};

export function statusMeta(code: string) {
  return STATUS_META[code] ?? { label: code, cls: "text-muted", badge: "badge-neutral" };
}

export function baseName(path: string) {
  const parts = path.split("/");
  return parts[parts.length - 1] || path;
}

export function dirName(path: string) {
  const idx = path.lastIndexOf("/");
  return idx > 0 ? path.slice(0, idx) : "";
}
