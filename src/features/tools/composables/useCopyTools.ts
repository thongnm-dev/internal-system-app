import { open } from "@tauri-apps/plugin-dialog";
import { onMounted, ref } from "vue";
import { tauriRuntimeMessage } from "@/shared/config/appConfig";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import {
  collectByFolders,
  collectLoadIni,
  collectRun,
  DEFAULT_COLLECT_CONFIG,
  type CollectConfig,
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

  async function run() {
    running.value = true;
    log.value = [];
    message.value = "Running collect...";
    messageMode.value = "info";
    try {
      const res = await collectRun(config.value);
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
    set,
    loadFromIni,
    pickFolder,
    run,
    runByFolders,
  };
}
