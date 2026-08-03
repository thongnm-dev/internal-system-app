import { open } from "@tauri-apps/plugin-dialog";
import { onMounted, ref } from "vue";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import {
  collectByFolders,
  collectLoadIni,
  collectRun,
  collectScanDuplicates,
  DEFAULT_COLLECT_CONFIG,
  type CollectConfig,
  type CollectDuplicateResult,
} from "@/tauri/commands/collect";
import type { MessageMode } from "@/_/types/app";

export function useCopyTools() {
  const config = ref<CollectConfig>({ ...DEFAULT_COLLECT_CONFIG });
  const running = ref(false);
  const runningFolders = ref(false);
  const log = ref<string[]>([]);
  const showResult = ref(false);
  const message = ref("Configure input/output paths, then run Copy or Copy by folder.");
  const messageMode = ref<MessageMode>("info");

  const duplicateResult = ref<CollectDuplicateResult | null>(null);
  const showDuplicateDialog = ref(false);
  const selectedDuplicates = ref<Set<string>>(new Set());

  function set<K extends keyof CollectConfig>(key: K, value: CollectConfig[K]) {
    config.value = { ...config.value, [key]: value };
  }

  async function loadFromIni(silent = false) {
    if (!canUseTauriRuntime()) {
      if (!silent) {
        message.value = tauriRuntimeMessage;
        messageMode.value = "error";
      }
      return;
    }
    try {
      config.value = await collectLoadIni();
      if (!silent) {
        message.value = "Loaded config from collect_input.ini.";
        messageMode.value = "info";
      }
    } catch (e) {
      if (!silent) {
        message.value = friendlyError(e);
        messageMode.value = "error";
      }
    }
  }

  async function pickFolder(key: "input" | "output") {
    if (!canUseTauriRuntime()) {
      message.value = tauriRuntimeMessage;
      messageMode.value = "error";
      return;
    }
    try {
      const selected = await open({ directory: true, multiple: false });
      if (typeof selected === "string") set(key, selected);
    } catch (e) {
      message.value = friendlyError(e);
      messageMode.value = "error";
    }
  }

  async function executeCollectRun(resolvedDuplicates: string[] = []) {
    running.value = true;
    log.value = [];
    message.value = "Running collect...";
    messageMode.value = "info";
    try {
      const cfg = { ...config.value, resolved_duplicates: resolvedDuplicates };
      const res = await collectRun(cfg);
      log.value = res.log;
      showResult.value = true;
      message.value = res.summary;
      messageMode.value = res.ok ? "info" : "error";
    } catch (e) {
      log.value = [`ERROR: ${String(e)}`];
      showResult.value = true;
      message.value = friendlyError(e);
      messageMode.value = "error";
    } finally {
      running.value = false;
    }
  }

  async function run() {
    if (config.value.use_newest) {
      await executeCollectRun();
      return;
    }

    running.value = true;
    message.value = "Scanning for duplicates...";
    messageMode.value = "info";
    try {
      const scan = await collectScanDuplicates(config.value);
      if (!scan.has_duplicates) {
        await executeCollectRun();
        return;
      }

      duplicateResult.value = scan;
      selectedDuplicates.value = new Set(
        scan.groups.map((g) => g.entries[0].path),
      );
      showDuplicateDialog.value = true;
      running.value = false;
    } catch (e) {
      log.value = [`ERROR: ${String(e)}`];
      showResult.value = true;
      message.value = friendlyError(e);
      messageMode.value = "error";
      running.value = false;
    }
  }

  async function runWithSelectedDuplicates() {
    showDuplicateDialog.value = false;
    await executeCollectRun(Array.from(selectedDuplicates.value));
  }

  function cancelDuplicateDialog() {
    showDuplicateDialog.value = false;
    duplicateResult.value = null;
    running.value = false;
    message.value = "Cancelled.";
    messageMode.value = "info";
  }

  function toggleDuplicateSelection(path: string, groupDest: string) {
    const group = duplicateResult.value?.groups.find((g) => g.dest === groupDest);
    if (!group) return;
    for (const entry of group.entries) {
      selectedDuplicates.value.delete(entry.path);
    }
    selectedDuplicates.value.add(path);
    selectedDuplicates.value = new Set(selectedDuplicates.value);
  }

  async function runByFolders() {
    runningFolders.value = true;
    log.value = [];
    message.value = "Running collect by folders...";
    messageMode.value = "info";
    try {
      const res = await collectByFolders(config.value);
      log.value = res.log;
      showResult.value = true;
      message.value = res.summary;
      messageMode.value = res.ok ? "info" : "error";
    } catch (e) {
      log.value = [`ERROR: ${String(e)}`];
      showResult.value = true;
      message.value = friendlyError(e);
      messageMode.value = "error";
    } finally {
      runningFolders.value = false;
    }
  }

  onMounted(() => loadFromIni(true));

  return {
    config,
    running,
    runningFolders,
    log,
    showResult,
    message,
    messageMode,
    duplicateResult,
    showDuplicateDialog,
    selectedDuplicates,
    set,
    loadFromIni,
    pickFolder,
    run,
    runByFolders,
    runWithSelectedDuplicates,
    cancelDuplicateDialog,
    toggleDuplicateSelection,
  };
}
