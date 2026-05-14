import { Clock3, MonitorCog, Network, UserRound } from "lucide-react";
import type { SystemInfo } from "../types/statistics";

type BottomBarProps = {
  info: SystemInfo;
};

export function BottomBar({ info }: BottomBarProps) {
  const items = [
    { label: "Login", value: info.username, icon: UserRound },
    { label: "Date time", value: formatDateTime(info.timestamp), icon: Clock3 },
    { label: "IP", value: info.ip_address, icon: Network },
  ];
  const VersionIcon = MonitorCog;

  return (
    <footer className="flex items-center gap-6 overflow-hidden border-t border-slate-950 bg-slate-800 px-4 py-2 text-sm text-slate-300">
      {items.map((item) => {
        const Icon = item.icon;
        return (
          <span key={item.label} className="status-item flex items-center gap-2" title={item.label}>
            <Icon className="h-4 w-4 shrink-0 text-emerald-300" />
            <strong className="min-w-0 truncate text-white">{item.value}</strong>
          </span>
        );
      })}
      <span className="status-item ml-auto flex items-center gap-2" title="Version">
        <VersionIcon className="h-4 w-4 shrink-0 text-emerald-300" />
        <strong className="min-w-0 truncate text-white">{info.version}</strong>
      </span>
    </footer>
  );
}

function formatDateTime(value: string) {
  const match = value.match(/^(\d{4})-(\d{2})-(\d{2}) (\d{2}:\d{2}:\d{2})$/);
  if (match) {
    return `${match[1]}/${match[2]}/${match[3]} ${match[4]}`;
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  const pad = (part: number) => part.toString().padStart(2, "0");
  return `${date.getFullYear()}/${pad(date.getMonth() + 1)}/${pad(date.getDate())} ${pad(date.getHours())}:${pad(
    date.getMinutes(),
  )}:${pad(date.getSeconds())}`;
}
