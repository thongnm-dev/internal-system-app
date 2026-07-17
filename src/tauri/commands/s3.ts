import { safeInvoke } from "./_base";
import type { AwsStorage, DeleteUploadedItem, LocalFileEntry, S3ListResult, S3OperationResult, ScannedFile, UploadFileRequest } from "@/_/types/s3";

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
