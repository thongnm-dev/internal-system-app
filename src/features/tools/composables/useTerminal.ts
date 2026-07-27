import { ref, shallowRef } from "vue";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { useToast } from "@/shared/composables/useToast";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { terminalKill, terminalResize, terminalSpawn, terminalWrite } from "@/tauri/commands/terminal";

/** Một tab terminal hiển thị trên UI (phần reactive, nhẹ). */
export interface TerminalTab {
  /** Định danh cục bộ, ổn định cho `v-for` và map instance xterm. */
  key: string;
  /** Id phiên PTY ở backend, có sau khi spawn thành công. */
  sessionId: string | null;
  /** Tên hiển thị trên tab. */
  title: string;
  /** Tiến trình shell đã kết thúc hay chưa. */
  exited: boolean;
  /** Mã thoát của tiến trình (nếu có). */
  exitCode: number | null;
}

/**
 * Instance xterm + phụ trợ cho một tab. Cố tình giữ NGOÀI reactive state của
 * Vue: xterm là object nặng, tự quản lý DOM/buffer riêng — bọc reactive sẽ làm
 * chậm và có thể gây lỗi. Chỉ `tabs` (mảng nhẹ) mới là reactive.
 */
interface TermEntry {
  term: Terminal;
  fit: FitAddon;
  ro: ResizeObserver | null;
  sessionId: string | null;
}

/** Giải mã base64 (byte thô từ PTY) thành `Uint8Array` để nạp vào xterm. */
function base64ToBytes(b64: string): Uint8Array {
  const bin = atob(b64);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return bytes;
}

/** Bảng màu tối cho terminal, đồng bộ với tông nền của app. */
const TERMINAL_THEME = {
  background: "#0b0f19",
  foreground: "#e5e9f0",
  cursor: "#e5e9f0",
  selectionBackground: "#33415580",
};

/**
 * Backend PTY trên Windows dùng ConPTY (qua `portable-pty`), vốn không phát ra
 * đúng chuỗi bao dòng (line-wrap) như PTY thật trên Unix. Nếu không khai báo
 * `windowsPty`, xterm hiểu sai việc bao dòng khi ứng dụng full-screen (vd. Claude
 * Code CLI) vẽ lại toàn màn hình, dẫn tới layout vỡ như đường viền box bị đứt/lệch.
 * Xem: https://github.com/xtermjs/xterm.js — option `windowsPty`.
 */
const IS_WINDOWS = navigator.userAgent.includes("Windows");

export function useTerminal() {
  const toast = useToast();

  const tabs = ref<TerminalTab[]>([]);
  const activeKey = ref<string>("");

  // Map instance xterm theo key tab (không reactive).
  const entries = new Map<string, TermEntry>();
  const startDir = shallowRef<string>("");

  let counter = 0;

  function nextKey(): string {
    counter += 1;
    return `tab-${Date.now()}-${counter}`;
  }

  /** Thêm một tab mới (chưa spawn — spawn diễn ra khi container được bind). */
  function addTab() {
    if (!canUseTauriRuntime()) {
      toast.error(tauriRuntimeMessage);
      return;
    }
    const key = nextKey();
    tabs.value.push({ key, sessionId: null, title: `Terminal ${tabs.value.length + 1}`, exited: false, exitCode: null });
    activeKey.value = key;
  }

  function setActive(key: string) {
    activeKey.value = key;
    // Refit khi chuyển tab vì container trước đó có thể bị ẩn (kích thước 0).
    requestAnimationFrame(() => fit(key));
  }

  /**
   * Gắn xterm vào phần tử DOM của tab. Gọi bởi component khi container sẵn sàng.
   * Idempotent: nếu tab đã gắn thì chỉ refit lại.
   */
  async function bindContainer(key: string, el: HTMLElement) {
    const existing = entries.get(key);
    if (existing) {
      existing.term.focus();
      fit(key);
      return;
    }

    const term = new Terminal({
      fontFamily: '"Cascadia Code", "JetBrains Mono", Menlo, Consolas, monospace',
      fontSize: 13,
      cursorBlink: true,
      scrollback: 5000,
      theme: TERMINAL_THEME,
      windowsPty: IS_WINDOWS ? { backend: "conpty", buildNumber: 0 } : undefined,
    });
    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(el);
    fitAddon.fit();

    const entry: TermEntry = { term, fit: fitAddon, ro: null, sessionId: null };
    entries.set(key, entry);

    // Gõ phím → gửi xuống PTY.
    term.onData((data) => {
      if (entry.sessionId) void terminalWrite(entry.sessionId, data).catch(() => undefined);
    });
    // xterm đổi số hàng/cột → báo backend resize PTY tương ứng.
    term.onResize(({ rows, cols }) => {
      if (entry.sessionId) void terminalResize(entry.sessionId, rows, cols).catch(() => undefined);
    });

    // Theo dõi thay đổi kích thước container để tự fit.
    const ro = new ResizeObserver(() => fit(key));
    ro.observe(el);
    entry.ro = ro;

    try {
      const sessionId = await terminalSpawn(
        term.rows,
        term.cols,
        // Output đến qua Channel riêng của phiên → nạp thẳng vào đúng terminal.
        (dataBase64) => term.write(base64ToBytes(dataBase64)),
        (code) => {
          const tab = tabs.value.find((t) => t.key === key);
          if (tab) {
            tab.exited = true;
            tab.exitCode = code;
          }
          term.writeln(`\r\n\x1b[90m[process exited${code === null ? "" : ` with code ${code}`}]\x1b[0m`);
        },
        startDir.value || undefined,
      );
      entry.sessionId = sessionId;
      const tab = tabs.value.find((t) => t.key === key);
      if (tab) tab.sessionId = sessionId;
      // Container có thể đã fit lại (đúng kích thước) trong lúc chờ spawn — lúc đó
      // sessionId còn null nên onResize không kịp báo cho backend. Đồng bộ lại ngay
      // để PTY khớp với kích thước xterm hiện tại, tránh app con vẽ theo cols cũ (hẹp).
      void terminalResize(sessionId, term.rows, term.cols).catch(() => undefined);
    } catch (e) {
      term.writeln(`\x1b[31m${friendlyError(e)}\x1b[0m`);
    }
  }

  /** Fit lại terminal của tab theo kích thước container hiện tại. */
  function fit(key: string) {
    const entry = entries.get(key);
    if (!entry) return;
    try {
      entry.fit.fit();
    } catch {
      // Container có thể đang ẩn (kích thước 0) — bỏ qua, sẽ fit lại khi hiện.
    }
  }

  /** Đóng một tab: kill PTY, giải phóng xterm, gỡ observer và cập nhật danh sách. */
  async function closeTab(key: string) {
    const entry = entries.get(key);
    if (entry) {
      entry.ro?.disconnect();
      if (entry.sessionId) await terminalKill(entry.sessionId).catch(() => undefined);
      entry.term.dispose();
      entries.delete(key);
    }

    const idx = tabs.value.findIndex((t) => t.key === key);
    if (idx !== -1) tabs.value.splice(idx, 1);

    if (activeKey.value === key) {
      const fallback = tabs.value[idx] ?? tabs.value[idx - 1] ?? tabs.value[0];
      activeKey.value = fallback ? fallback.key : "";
      if (fallback) requestAnimationFrame(() => fit(fallback.key));
    }
  }

  /** Đặt thư mục làm việc mặc định cho các tab mở sau. */
  function setStartDir(dir: string) {
    startDir.value = dir;
  }

  /** Mở sẵn 1 tab đầu tiên nếu đang chạy trong Tauri runtime. */
  function init() {
    if (!canUseTauriRuntime()) return;
    if (tabs.value.length === 0) addTab();
  }

  /** Dọn dẹp toàn bộ: kill + dispose mọi phiên terminal. */
  async function dispose() {
    for (const [key, entry] of entries) {
      entry.ro?.disconnect();
      if (entry.sessionId) await terminalKill(entry.sessionId).catch(() => undefined);
      entry.term.dispose();
      entries.delete(key);
    }
    tabs.value = [];
    activeKey.value = "";
  }

  return {
    tabs,
    activeKey,
    addTab,
    setActive,
    bindContainer,
    closeTab,
    setStartDir,
    startDir,
    fit,
    init,
    dispose,
  };
}
