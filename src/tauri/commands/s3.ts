import { safeInvoke } from "./_base";
import type { S3ListResult, S3OperationResult } from "@/_/types/s3";

export function s3TestConnection() {
  return safeInvoke<string>("s3_test_connection");
}

export function s3ListObjects(prefix: string) {
  return safeInvoke<S3ListResult>("s3_list_objects", { prefix });
}

export function s3DownloadObjects(keys: string[], destinationDir: string) {
  return safeInvoke<S3OperationResult>("s3_download_objects", { keys, destinationDir });
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
