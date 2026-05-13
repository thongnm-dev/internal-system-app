import type { SystemInfo } from "../types/statistics";

type BottomBarProps = {
  info: SystemInfo;
};

export function BottomBar({ info }: BottomBarProps) {
  return (
    <footer className="grid grid-cols-4 gap-3 border-t border-slate-950 bg-slate-800 px-4 py-2 text-sm text-slate-300">
      <span className="status-item">
        Login: <strong className="text-white">{info.username}</strong>
      </span>
      <span className="status-item">
        Date time: <strong className="text-white">{info.timestamp}</strong>
      </span>
      <span className="status-item">
        IP: <strong className="text-white">{info.ip_address}</strong>
      </span>
      <span className="status-item">
        Version: <strong className="text-white">{info.version}</strong>
      </span>
    </footer>
  );
}
