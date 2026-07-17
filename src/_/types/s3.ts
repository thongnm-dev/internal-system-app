export interface S3Config {
  accessKeyId: string;
  secretAccessKey: string;
  region: string;
  bucket: string;
  endpointUrl: string | null;
}

export interface S3Object {
  key: string;
  displayName: string;
  size: number;
  lastModified: string;
  isFolder: boolean;
  etag: string;
}

export interface S3ListResult {
  objects: S3Object[];
  currentPrefix: string;
}

export interface S3OperationResult {
  success: boolean;
  message: string;
  processed: number;
  failed: number;
}

export interface AwsStorage {
  id: number;
  code: string;
  name: string;
  nameAlias: string;
  subscribe: string;
  isUpload: boolean;
  isDownload: boolean;
  fileOnly: boolean;
  linkAvailable: string[];
  excludeSubscribe: string[];
}

export interface UploadFileRequest {
  parentName: string;
  name: string;
  localPath: string;
}

export interface ScannedFile {
  parentName: string;
  name: string;
  filePath: string;
  fullPath: string;
  fileSize: number;
}

export interface DeleteUploadedItem {
  awsCd: string;
  bugNo: string;
}
