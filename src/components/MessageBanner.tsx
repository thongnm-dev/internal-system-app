import type { MessageMode } from "../types/statistics";

type MessageBannerProps = {
  message: string;
  mode: MessageMode;
};

export function MessageBanner({ message, mode }: MessageBannerProps) {
  return (
    <div
      className={[
        "border-l-4 bg-white px-4 py-2.5 text-sm shadow-sm",
        mode === "error" ? "border-red-600 text-red-800" : "border-brand text-slate-700",
      ].join(" ")}
    >
      {message}
    </div>
  );
}
