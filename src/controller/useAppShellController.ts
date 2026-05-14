import { useEffect, useState } from "react";
import { friendlyError, safeInvoke } from "../core/tauriRuntime";
import { useHashRouter } from "../router/useHashRouter";
import type { MessageMode, SelectedPhaseDetail, SystemInfo } from "../types/statistics";

export function useAppShellController() {
  const router = useHashRouter();
  const [systemInfo, setSystemInfo] = useState<SystemInfo>({
    username: "-",
    timestamp: "-",
    ip_address: "-",
    version: "-",
  });
  const [message, setMessage] = useState("Ready to read CSV data.");
  const [messageMode, setMessageMode] = useState<MessageMode>("info");
  const [selectedPhaseDetail, setSelectedPhaseDetail] = useState<SelectedPhaseDetail | null>(null);
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);

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
    void refreshSystemInfo();
    const timer = window.setInterval(() => void refreshSystemInfo(), 1000);
    return () => window.clearInterval(timer);
  }, []);

  return {
    ...router,
    isSidebarCollapsed,
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
