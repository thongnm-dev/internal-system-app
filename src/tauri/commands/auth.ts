import { safeInvoke } from "./_base";
import type { LoginRequest, LoginResponse } from "@/_/types/auth";

export function login(request: LoginRequest) {
  return safeInvoke<LoginResponse>("login", { request });
}
