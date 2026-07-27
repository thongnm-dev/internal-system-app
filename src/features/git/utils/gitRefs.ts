import type { GitBranch } from "@/_/types/git";

/** Đoán branch base hợp lý nhất (main/master/develop…) để so sánh/PR/merge. */
export function guessBase(branchNames: string[], head: string, upstream?: string): string {
  for (const cand of ["origin/main", "origin/master", "main", "master", "develop"]) {
    if (cand !== head && branchNames.includes(cand)) return cand;
  }
  return upstream || branchNames.find((n) => n !== head) || "";
}

/** Suy ra ref đầy đủ (ưu tiên remote) cho một tên branch ngắn (vd. từ PR). */
export function resolveRef(branches: GitBranch[], name: string): string {
  if (branches.some((b) => b.is_remote && b.name === `origin/${name}`)) return `origin/${name}`;
  if (branches.some((b) => !b.is_remote && b.name === name)) return name;
  return `origin/${name}`;
}
