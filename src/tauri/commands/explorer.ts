import { safeInvoke } from "./_base";

export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified: string;
  extension: string;
}

export interface ReadDirResult {
  path: string;
  entries: FileEntry[];
}

export interface SearchResult {
  query: string;
  root: string;
  entries: FileEntry[];
  truncated: boolean;
}

export function explorerReadDir(path: string) {
  return safeInvoke<ReadDirResult>("explorer_read_dir", { path });
}

export function explorerSearch(root: string, query: string) {
  return safeInvoke<SearchResult>("explorer_search", { root, query });
}

export function explorerOpen(path: string) {
  return safeInvoke<void>("explorer_open", { path });
}

export function explorerGetDrives() {
  return safeInvoke<string[]>("explorer_get_drives");
}

export function explorerRename(path: string, newName: string) {
  return safeInvoke<void>("explorer_rename", { path, newName });
}

export function explorerDelete(paths: string[]) {
  return safeInvoke<void>("explorer_delete", { paths });
}

export function explorerCreateFile(dir: string, name: string) {
  return safeInvoke<string>("explorer_create_file", { dir, name });
}

export function explorerCreateFolder(dir: string, name: string) {
  return safeInvoke<string>("explorer_create_folder", { dir, name });
}

export function explorerPaste(sources: string[], destDir: string, cut: boolean) {
  return safeInvoke<void>("explorer_paste", { sources, destDir, cut });
}

export function explorerCopyBugs(sourceDir: string, destDir: string) {
  return safeInvoke<string>("explorer_copy_bugs", { sourceDir, destDir });
}
