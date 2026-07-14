export type DatabaseStatus = {
  configured: boolean;
  connected: boolean;
  message: string;
};

export type DatabaseConfig = {
  host: string;
  port: number;
  dbname: string;
  user: string;
  password: string;
};

export type SaveDatabaseConfigRequest = {
  host: string;
  port: number;
  dbname: string;
  user: string;
  password: string;
};
