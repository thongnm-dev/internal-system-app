import { useEffect, useState } from "react";
import { friendlyError, safeInvoke } from "../core/tauriRuntime";
import { useHashRouter } from "../router/useHashRouter";
import type { MessageMode, SelectedPhaseDetail, SystemInfo } from "../types/statistics";
import { applyStoredThemePreference } from "./useSettingsController";

const BOOTSTRAP_MIN_DURATION_MS = 900;

async function setCurrentWindowResizable(isResizable: boolean) {
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().setResizable(isResizable);
  } catch {
    // Browser dev sessions do not expose a Tauri window.
  }
}

export function useAppShellController() {
  const router = useHashRouter();
  const [systemInfo, setSystemInfo] = useState<SystemInfo>({
    username: "-",
    timestamp: "-",
    ip_address: "-",
    version: "-",
  });
  const [message, setMessage] = useState("");
  const [messageMode, setMessageMode] = useState<MessageMode>("info");
  const [selectedPhaseDetail, setSelectedPhaseDetail] = useState<SelectedPhaseDetail | null>(null);
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);
  const [isBootstrapping, setIsBootstrapping] = useState(true);

  const refreshSystemInfo = async () => {
    try {
      const info = await safeInvoke<SystemInfo>("get_system_info");
      setSystemInfo(info);
    } catch {
      setSystemInfo((current) => ({
        ...current,
        timestamp: new Date().toLocaleString(),
      }));
    }
  };

  const showMessage = (nextMessage: string, nextMode: MessageMode = "info") => {
    setMessage(nextMessage);
    setMessageMode(nextMode);
  };

  const showError = (error: unknown) => {
    showMessage(friendlyError(error), "error");
  };

  useEffect(() => {
    let isMounted = true;
    let timer: number | undefined;
    const startedAt = window.performance.now();

    const bootstrap = async () => {
      await setCurrentWindowResizable(false);
      applyStoredThemePreference();
      await refreshSystemInfo();

      const elapsed = window.performance.now() - startedAt;
      const remainingDelay = Math.max(0, BOOTSTRAP_MIN_DURATION_MS - elapsed);

      window.setTimeout(() => {
        if (!isMounted) {
          return;
        }

        setIsBootstrapping(false);
        void setCurrentWindowResizable(true);
        timer = window.setInterval(() => void refreshSystemInfo(), 1000);
      }, remainingDelay);
    };

    void bootstrap();

    return () => {
      isMounted = false;
      if (timer !== undefined) {
        window.clearInterval(timer);
      }
      void setCurrentWindowResizable(true);
    };
  }, []);

  return {
    ...router,
    isSidebarCollapsed,
    isBootstrapping,
    message,
    messageMode,
    selectedPhaseDetail,
    setIsSidebarCollapsed,
    setSelectedPhaseDetail,
    showError,
    showMessage,
    systemInfo,
  };
}
