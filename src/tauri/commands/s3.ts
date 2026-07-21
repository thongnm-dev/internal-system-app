import { safeInvoke } from "./_base";
import type { AwsStorage, DeleteUploadedItem, DownloadAvailability, LocalFileEntry, S3ListResult, S3OperationResult, ScannedFile, UploadFileRequest } from "@/_/types/s3";

export type S3Config = {
  accessKeyId: string;
  secretAccessKey: string;
  region: string;
  bucket: string;
  endpointUrl?: string;
};

export function s3CheckConfig() {
  return safeInvoke<void>("s3_check_config");
}

export function s3GetConfig() {
  return safeInvoke<S3Config>("s3_get_config");
}

export function s3SaveConfig(config: S3Config) {
  return safeInvoke<void>("s3_save_config", { config });
}

export function s3TestConnection() {
  return safeInvoke<string>("s3_test_connection");
}

export function s3ListObjects(prefix: string) {
  return safeInvoke<S3ListResult>("s3_list_objects", { prefix });
}

export function s3DownloadObjects(keys: string[], destinationDir: string, stripPrefix: string) {
  return safeInvoke<S3OperationResult>("s3_download_objects", { keys, destinationDir, stripPrefix });
}

export function s3UploadFile(localPath: string, s3Key: string) {
  return safeInvoke<S3OperationResult>("s3_upload_file", { localPath, s3Key });
}

export function s3DeleteObjects(keys: string[]) {
  return safeInvoke<S3OperationResult>("s3_delete_objects", { keys });
}

export function s3CreateFolder(prefix: string) {
  return safeInvoke<S3OperationResult>("s3_create_folder", { prefix });
}

export function s3ListUploadStorages() {
  return safeInvoke<AwsStorage[]>("s3_list_upload_storages");
}

export function s3ScanUploadFolder(dirPath: string) {
  return safeInvoke<ScannedFile[]>("s3_scan_upload_folder", { dirPath });
}

export function s3UploadFiles(files: UploadFileRequest[], storageName: string, subscribe: string, createFolderSameName: boolean) {
  return safeInvoke<S3OperationResult>("s3_upload_files", { files, storageName, subscribe, createFolderSameName });
}

export function s3ListDeleteOptions(destinationCode: string) {
  return safeInvoke<AwsStorage[]>("s3_list_delete_options", { destinationCode });
}

export function s3ScanLocalFolder(folderPath: string) {
  return safeInvoke<LocalFileEntry[]>("s3_scan_local_folder", { folderPath });
}

export function s3UploadFolderToS3(folderPath: string, s3Prefix: string) {
  return safeInvoke<S3OperationResult>("s3_upload_folder", { folderPath, s3Prefix });
}

export function s3DeleteUploadedItems(items: DeleteUploadedItem[]) {
  return safeInvoke<S3OperationResult>("s3_delete_uploaded_items", { items });
}

export function s3ListDownloadStorages() {
  return safeInvoke<AwsStorage[]>("s3_list_download_storages");
}

export function s3CheckDownloadAvailable(codes: string[]) {
  return safeInvoke<Record<string, DownloadAvailability>>("s3_check_download_available", { codes });
}

export function s3GetDownloadList(code: string) {
  return safeInvoke<string[]>("s3_get_download_list", { code });
}

export function s3DownloadByStorage(code: string, bugList: string[], localPath: string) {
  return safeInvoke<S3OperationResult>("s3_download_by_storage", { code, bugList, localPath });
}

export function s3MoveObjects(code: string, items: string[]) {
  return safeInvoke<S3OperationResult>("s3_move_objects", { code, items });
}

export function s3MoveBrowserObjects(keys: string[], destinationPrefix: string) {
  return safeInvoke<S3OperationResult>("s3_move_browser_objects", { keys, destinationPrefix });
}

export function s3DeleteByStorage(code: string, items: string[]) {
  return safeInvoke<S3OperationResult>("s3_delete_by_storage", { code, items });
}

export function s3GetBrowserAllowedPrefixes() {
  return safeInvoke<string[]>("s3_get_browser_allowed_prefixes");
}
