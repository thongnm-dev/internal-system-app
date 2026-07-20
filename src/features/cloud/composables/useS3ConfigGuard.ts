import { ref } from "vue";
import { canUseTauriRuntime } from "@/tauri/commands/_base";
import { s3CheckConfig } from "@/tauri/commands/s3";

const configError = ref("");
const configChecking = ref(false);
const configReady = ref(false);

export function useS3ConfigGuard() {
  async function checkConfig() {
    if (!canUseTauriRuntime()) return;
    configChecking.value = true;
    configError.value = "";
    try {
      await s3CheckConfig();
      configReady.value = true;
    } catch (e) {
      configError.value = String(e);
      configReady.value = false;
    } finally {
      configChecking.value = false;
    }
  }

  return { configError, configChecking, configReady, checkConfig };
}
