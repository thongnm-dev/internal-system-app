import { computed, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useToast } from "@/shared/composables/useToast";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { aiUsageListAccounts, aiUsageOpenTerminal, aiUsageSetActive } from "@/tauri/commands/ai-usage";
import type { AiAccount } from "@/_/types/ai-usage";
import {
  explorerCreateFolder,
  explorerDelete,
  explorerEnsureDir,
  explorerOpen,
  explorerPaste,
  explorerReadDir,
  explorerReadTextFile,
} from "@/tauri/commands/explorer";
import type { FileEntry } from "@/tauri/commands/explorer";

export type Clipboard = { paths: string[]; cut: boolean } | null;

function parentOf(path: string): string {
  const normalized = path.replace(/[/\\]+$/, "");
  const idx = Math.max(normalized.lastIndexOf("/"), normalized.lastIndexOf("\\"));
  return idx > 0 ? normalized.slice(0, idx) : "";
}

export function useAiTranslateCowork() {
  const toast = useToast();

  // Row 1 — project directory.
  const projectDir = ref("");
  const dirEntries = ref<FileEntry[]>([]);
  const isLoadingDir = ref(false);

  // Row 2 — AI accounts.
  const accounts = ref<AiAccount[]>([]);
  const isLoadingAccounts = ref(false);
  const settingActiveId = ref<number | null>(null);

  // Column 2 — skills discovered under `<projectDir>/.claude/skills`.
  const skillFolders = ref<string[]>([]);
  const isLoadingSkills = ref(false);
  const openingSkillTerminal = ref<string | null>(null);

  // Column 3 — project directory listing markdown preview dialog.
  const mdPreviewOpen = ref(false);
  const mdPreviewName = ref("");
  const mdPreviewContent = ref("");
  const isLoadingMdPreview = ref(false);

  // Clipboard shared between the input and output folder panels.
  const clipboard = ref<Clipboard>(null);

  const activeAccount = computed(
    () => accounts.value.find((a) => a.is_active && a.config_dir.trim()) ?? null,
  );

  /**
   * A browsable folder panel rooted at `<projectDir>/<subfolder>` (e.g. `input`, `output`),
   * with new-folder / copy / paste / delete operations. Both panels share one clipboard.
   */
  function makeFolderPanel(subfolder: string) {
    const rootDir = ref("");
    const dir = ref("");
    const entries = ref<FileEntry[]>([]);
    const isLoading = ref(false);
    const selected = ref<Set<string>>(new Set());

    const isAtRoot = computed(() => !rootDir.value || dir.value === rootDir.value);

    async function load() {
      const target = dir.value.trim();
      if (!target) {
        entries.value = [];
        return;
      }
      isLoading.value = true;
      try {
        const result = await explorerReadDir(target);
        dir.value = result.path;
        entries.value = result.entries;
        selected.value = new Set();
      } catch (e) {
        toast.error(friendlyError(e));
      } finally {
        isLoading.value = false;
      }
    }

    /** Point the panel at `<projectPath>/<subfolder>`, creating the folder if missing. */
    async function setRoot(projectPath: string) {
      let target = `${projectPath}/${subfolder}`;
      try {
        // Idempotent mkdir -p — no error when the folder already exists.
        target = await explorerEnsureDir(target);
      } catch (e) {
        toast.error(friendlyError(e));
      }
      dir.value = target;
      await load();
      // `load()` normalises the path; anchor the root to the normalised form.
      rootDir.value = dir.value;
    }

    function reset() {
      rootDir.value = "";
      dir.value = "";
      entries.value = [];
      selected.value = new Set();
    }

    async function openEntry(entry: FileEntry) {
      if (entry.is_dir) {
        dir.value = entry.path;
        await load();
      }
    }

    async function goUp() {
      if (isAtRoot.value) return;
      const parent = parentOf(dir.value);
      if (!parent) return;
      dir.value = parent;
      await load();
    }

    function isSelected(entry: FileEntry): boolean {
      return selected.value.has(entry.path);
    }

    function toggleSelected(entry: FileEntry) {
      const next = new Set(selected.value);
      if (next.has(entry.path)) next.delete(entry.path);
      else next.add(entry.path);
      selected.value = next;
    }

    async function createFolder(name: string) {
      const target = dir.value.trim();
      const folderName = name.trim();
      if (!target || !folderName) return;
      try {
        await explorerCreateFolder(target, folderName);
        await load();
        toast.success("Folder created.");
      } catch (e) {
        toast.error(friendlyError(e));
      }
    }

    function copySelected() {
      const paths = Array.from(selected.value);
      if (!paths.length) return;
      clipboard.value = { paths, cut: false };
      toast.success(`${paths.length} item(s) copied.`);
    }

    async function paste() {
      if (!clipboard.value) return;
      const target = dir.value.trim();
      if (!target) return;
      try {
        await explorerPaste(clipboard.value.paths, target, clipboard.value.cut);
        if (clipboard.value.cut) clipboard.value = null;
        await load();
        toast.success("Pasted.");
      } catch (e) {
        toast.error(friendlyError(e));
      }
    }

    async function deleteSelected() {
      const paths = Array.from(selected.value);
      if (!paths.length) return;
      try {
        await explorerDelete(paths);
        await load();
        toast.success(`${paths.length} item(s) deleted.`);
      } catch (e) {
        toast.error(friendlyError(e));
      }
    }

    return {
      rootDir,
      dir,
      entries,
      isLoading,
      selected,
      isAtRoot,
      load,
      setRoot,
      reset,
      openEntry,
      goUp,
      isSelected,
      toggleSelected,
      createFolder,
      copySelected,
      paste,
      deleteSelected,
    };
  }

  const input = makeFolderPanel("input");
  const output = makeFolderPanel("output");

  // --- Row 1: project directory ---
  async function pickProjectDir() {
    if (!canUseTauriRuntime()) {
      toast.error(tauriRuntimeMessage);
      return;
    }
    try {
      const selected = await open({ directory: true, title: "Choose project directory" });
      if (typeof selected === "string") {
        projectDir.value = selected;
        await loadDirectory();
      }
    } catch (e) {
      toast.error(friendlyError(e));
    }
  }

  function clearProjectDir() {
    projectDir.value = "";
    dirEntries.value = [];
    skillFolders.value = [];
    clipboard.value = null;
    input.reset();
    output.reset();
  }

  async function loadDirectory() {
    const dir = projectDir.value.trim();
    if (!dir) {
      dirEntries.value = [];
      return;
    }
    isLoadingDir.value = true;
    try {
      const result = await explorerReadDir(dir);
      projectDir.value = result.path;
      dirEntries.value = result.entries;
      // Re-root the input/output panels + reload skills on every project (re)load.
      await Promise.all([input.setRoot(result.path), output.setRoot(result.path), loadSkillFolders()]);
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      isLoadingDir.value = false;
    }
  }

  async function showInFolder(path: string) {
    try {
      await explorerOpen(path);
    } catch (e) {
      toast.error(friendlyError(e));
    }
  }

  // --- Row 2: AI accounts ---
  async function loadAccounts() {
    isLoadingAccounts.value = true;
    try {
      accounts.value = await aiUsageListAccounts();
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      isLoadingAccounts.value = false;
    }
  }

  async function setActiveAccount(id: number) {
    settingActiveId.value = id;
    try {
      await aiUsageSetActive(id);
      await loadAccounts();
      toast.success("Account switched.");
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      settingActiveId.value = null;
    }
  }

  // --- Column 2: skills ---
  async function loadSkillFolders() {
    const dir = projectDir.value.trim();
    if (!dir) {
      skillFolders.value = [];
      return;
    }
    isLoadingSkills.value = true;
    try {
      const result = await explorerReadDir(`${dir}/.claude/skills`);
      skillFolders.value = result.entries.filter((e) => e.is_dir).map((e) => e.name);
    } catch {
      skillFolders.value = [];
    } finally {
      isLoadingSkills.value = false;
    }
  }

  /** Mở terminal tại project directory với account AI đang active để chạy skill. */
  async function openSkillTerminal(name: string) {
    const dir = projectDir.value.trim();
    if (!dir) return;
    const active = activeAccount.value;
    if (!active) {
      toast.error("Không tìm thấy account AI đang active có CLAUDE_CONFIG_DIR.");
      return;
    }
    openingSkillTerminal.value = name;
    try {
      await aiUsageOpenTerminal(active.config_dir, dir);
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      openingSkillTerminal.value = null;
    }
  }

  // --- Column 3: markdown preview dialog ---
  async function openMarkdownPreview(entry: FileEntry) {
    mdPreviewName.value = entry.name;
    mdPreviewContent.value = "";
    mdPreviewOpen.value = true;
    isLoadingMdPreview.value = true;
    try {
      mdPreviewContent.value = await explorerReadTextFile(entry.path);
    } catch (e) {
      toast.error(friendlyError(e));
      mdPreviewOpen.value = false;
    } finally {
      isLoadingMdPreview.value = false;
    }
  }

  async function init() {
    await loadAccounts();
  }

  return {
    // row 1
    projectDir,
    dirEntries,
    isLoadingDir,
    pickProjectDir,
    clearProjectDir,
    loadDirectory,
    showInFolder,
    // row 2
    accounts,
    isLoadingAccounts,
    settingActiveId,
    activeAccount,
    loadAccounts,
    setActiveAccount,
    // col 1 — input / output folder panels
    clipboard,
    input,
    output,
    // col 2
    skillFolders,
    isLoadingSkills,
    openingSkillTerminal,
    loadSkillFolders,
    openSkillTerminal,
    // col 3
    mdPreviewOpen,
    mdPreviewName,
    mdPreviewContent,
    isLoadingMdPreview,
    openMarkdownPreview,
    // lifecycle
    init,
  };
}
