import { safeInvoke } from "./_base";
import type { AiCoworkState } from "@/_/types/ai-cowork";

export function aiCoworkGetState() {
  return safeInvoke<AiCoworkState>("ai_cowork_get_state");
}

export function aiCoworkSaveState(state: AiCoworkState) {
  return safeInvoke<void>("ai_cowork_save_state", { state });
}
