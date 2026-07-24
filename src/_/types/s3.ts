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

export interface StorageBugFolders {
  storage: AwsStorage;
  bugs: string[];
}

export interface BugFolderItem {
  bugNo: string;
  inSubscribe: boolean;
  lastModified: string;
}

export interface BugFolderTab {
  name: string;
  nameAlias: string;
  items: BugFolderItem[];
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

export interface LocalFileEntry {
  name: string;
  relativePath: string;
  fullPath: string;
  size: number;
}

export interface DeleteUploadedItem {
  awsCd: string;
  bugNo: string;
}

export interface DownloadAvailability {
  downloadAvailable: boolean;
}

export interface DownloadByStorageResult {
  success: boolean;
  message: string;
  processed: number;
  failed: number;
  syncPath: string;
}

export interface DownloadHistorySearchParams {
  fromDate: string;
  toDate: string;
  awsCd: string;
  bugNo: string;
  isMovedAtLocal: boolean;
  isMovedAtS3: boolean;
}

export interface DownloadHistorySearchItem {
  id: number;
  downloadYmd: string;
  awsCd: string;
  awsName: string;
  isMovedAtLocal: boolean;
  bugNo: string;
}

export interface UploadHistorySearchParams {
  fromDate: string;
  toDate: string;
  awsCd: string;
  bugNo: string;
  isMovedAtS3: boolean;
}

export interface UploadHistorySearchItem {
  uploadYmd: string;
  awsName: string;
  isMovedAtS3: boolean;
  bugNo: string;
  attFiles: string;
}

export interface DownloadHistoryItem {
  id: number;
  downloadYmd: string;
  downloadHms: string;
  syncPath: string;
  downloadCount: number;
  isMovedAtLocal: boolean;
  awsName: string;
  awsNameAlias: string;
}
