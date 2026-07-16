import { safeInvoke } from "./_base";
import type { S3Config, S3ListResult, S3OperationResult } from "@/shared/types/s3";

export function s3LoadConfig() {
  return safeInvoke<S3Config>("s3_load_config");
}

export function s3TestConnection(config: S3Config) {
  return safeInvoke<string>("s3_test_connection", { config });
}

export function s3ListObjects(config: S3Config, prefix: string) {
  return safeInvoke<S3ListResult>("s3_list_objects", { config, prefix });
}

export function s3DownloadObjects(config: S3Config, keys: string[], destinationDir: string) {
  return safeInvoke<S3OperationResult>("s3_download_objects", { config, keys, destinationDir });
}

export function s3UploadFile(config: S3Config, localPath: string, s3Key: string) {
  return safeInvoke<S3OperationResult>("s3_upload_file", { config, localPath, s3Key });
}

export function s3DeleteObjects(config: S3Config, keys: string[]) {
  return safeInvoke<S3OperationResult>("s3_delete_objects", { config, keys });
}

export function s3CreateFolder(config: S3Config, prefix: string) {
  return safeInvoke<S3OperationResult>("s3_create_folder", { config, prefix });
}
