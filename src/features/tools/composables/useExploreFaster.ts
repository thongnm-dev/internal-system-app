import { ref, computed, watch } from "vue";
import {
  explorerReadDir,
  explorerSearch,
  explorerOpen,
  explorerGetDrives,
  type FileEntry,
} from "@/tauri/commands/explorer";
import { friendlyError } from "@/tauri/commands/_base";

export function useExploreFaster() {
  const currentPath = ref("");
  const entries = ref<FileEntry[]>([]);
  const searchQuery = ref("");
  const searchResults = ref<FileEntry[]>([]);
  const searchTruncated = ref(false);
  const isSearching = ref(false);
  const isLoading = ref(false);
  const error = ref("");
  const drives = ref<string[]>([]);
  const history = ref<string[]>([]);
  const historyIndex = ref(-1);

  const isSearchMode = computed(() => searchQuery.value.trim().length > 0 && searchResults.value.length > 0 || isSearching.value);

  const breadcrumbs = computed(() => {
    const p = currentPath.value;
    if (!p) return [];

    const parts: { label: string; path: string }[] = [];
    const normalized = p.replace(/\//g, "\\");
    const segments = normalized.split("\\").filter(Boolean);

    if (segments.length === 0) return [];

    let accumulated = "";
    for (let i = 0; i < segments.length; i++) {
      if (i === 0 && segments[0].endsWith(":")) {
        accumulated = segments[0] + "\\";
        parts.push({ label: segments[0] + "\\", path: accumulated });
      } else {
        accumulated = accumulated + segments[i] + "\\";
        parts.push({ label: segments[i], path: accumulated });
      }
    }
    return parts;
  });

  async function loadDrives() {
    try {
      drives.value = await explorerGetDrives();
      if (drives.value.length > 0 && !currentPath.value) {
        await navigateTo(drives.value[0]);
      }
    } catch (e) {
      error.value = friendlyError(e);
    }
  }

  async function navigateTo(path: string) {
    isLoading.value = true;
    error.value = "";
    searchQuery.value = "";
    searchResults.value = [];
    try {
      const result = await explorerReadDir(path);
      currentPath.value = result.path;
      entries.value = result.entries;

      if (historyIndex.value < history.value.length - 1) {
        history.value = history.value.slice(0, historyIndex.value + 1);
      }
      history.value.push(result.path);
      historyIndex.value = history.value.length - 1;
    } catch (e) {
      error.value = friendlyError(e);
    } finally {
      isLoading.value = false;
    }
  }

  async function refresh() {
    if (!currentPath.value) return;
    isLoading.value = true;
    error.value = "";
    try {
      const result = await explorerReadDir(currentPath.value);
      entries.value = result.entries;
    } catch (e) {
      error.value = friendlyError(e);
    } finally {
      isLoading.value = false;
    }
  }

  async function goUp() {
    if (!currentPath.value) return;
    const normalized = currentPath.value.replace(/\//g, "\\").replace(/\\$/, "");
    const lastSlash = normalized.lastIndexOf("\\");
    if (lastSlash <= 0) return;

    const isDriveRoot = normalized.length === 3 && normalized[1] === ":" && normalized[2] === "\\";
    if (isDriveRoot) return;

    const parent = normalized.substring(0, lastSlash + 1);
    await navigateTo(parent);
  }

  function goBack() {
    if (historyIndex.value > 0) {
      historyIndex.value--;
      loadPathFromHistory();
    }
  }

  function goForward() {
    if (historyIndex.value < history.value.length - 1) {
      historyIndex.value++;
      loadPathFromHistory();
    }
  }

  async function loadPathFromHistory() {
    const path = history.value[historyIndex.value];
    if (!path) return;
    isLoading.value = true;
    error.value = "";
    searchQuery.value = "";
    searchResults.value = [];
    try {
      const result = await explorerReadDir(path);
      currentPath.value = result.path;
      entries.value = result.entries;
    } catch (e) {
      error.value = friendlyError(e);
    } finally {
      isLoading.value = false;
    }
  }

  let searchTimer: ReturnType<typeof setTimeout> | null = null;

  function triggerSearch() {
    if (searchTimer) clearTimeout(searchTimer);
    const q = searchQuery.value.trim();
    if (!q) {
      searchResults.value = [];
      searchTruncated.value = false;
      return;
    }
    searchTimer = setTimeout(() => doSearch(q), 300);
  }

  async function doSearch(query: string) {
    if (!currentPath.value) return;
    isSearching.value = true;
    error.value = "";
    try {
      const result = await explorerSearch(currentPath.value, query);
      if (searchQuery.value.trim() === query) {
        searchResults.value = result.entries;
        searchTruncated.value = result.truncated;
      }
    } catch (e) {
      error.value = friendlyError(e);
    } finally {
      isSearching.value = false;
    }
  }

  function clearSearch() {
    searchQuery.value = "";
    searchResults.value = [];
    searchTruncated.value = false;
  }

  async function openEntry(entry: FileEntry) {
    if (entry.is_dir) {
      await navigateTo(entry.path);
    } else {
      await openInExplorer(entry.path);
    }
  }

  async function openInExplorer(path: string) {
    try {
      await explorerOpen(path);
    } catch (e) {
      error.value = friendlyError(e);
    }
  }

  async function handlePathSubmit(newPath: string) {
    const trimmed = newPath.trim();
    if (!trimmed) return;
    await navigateTo(trimmed);
  }

  const canGoBack = computed(() => historyIndex.value > 0);
  const canGoForward = computed(() => historyIndex.value < history.value.length - 1);

  return {
    currentPath,
    entries,
    searchQuery,
    searchResults,
    searchTruncated,
    isSearching,
    isLoading,
    isSearchMode,
    error,
    drives,
    breadcrumbs,
    canGoBack,
    canGoForward,
    loadDrives,
    navigateTo,
    refresh,
    goUp,
    goBack,
    goForward,
    triggerSearch,
    clearSearch,
    openEntry,
    openInExplorer,
    handlePathSubmit,
  };
}
