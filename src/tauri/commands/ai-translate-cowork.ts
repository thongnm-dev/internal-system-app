import { safeInvoke } from "./_base";
import type { AiTranslateCoworkState } from "@/_/types/ai-translate-cowork";

export function aiTranslateCoworkGetState() {
  return safeInvoke<AiTranslateCoworkState>("ai_translate_cowork_get_state");
}

export function aiTranslateCoworkSaveState(projectDir: string) {
  return safeInvoke<void>("ai_translate_cowork_save_state", { projectDir });
}
