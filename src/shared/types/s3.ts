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
