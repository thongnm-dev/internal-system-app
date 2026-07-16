import { safeInvoke } from "./_base";
import type { AddAiAccountRequest, AiAccount } from "@/_/types/ai-usage";

export function aiUsageAddAccount(request: AddAiAccountRequest) {
  return safeInvoke<AiAccount>("ai_usage_add_account", { request });
}

export function aiUsageListAccounts() {
  return safeInvoke<AiAccount[]>("ai_usage_list_accounts");
}

export function aiUsageDeleteAccount(id: number) {
  return safeInvoke<void>("ai_usage_delete_account", { id });
}
