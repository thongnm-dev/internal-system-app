export type SqlConnection = {
  id: number;
  name: string;
  db_type: string;
  host: string;
  port: number;
  database: string;
  username: string;
  password: string;
  created_at: string;
};

export type SaveSqlConnectionRequest = {
  id?: number;
  name: string;
  db_type: string;
  host: string;
  port: number;
  database: string;
  username: string;
  password: string;
};

export type DbTable = {
  schema: string;
  name: string;
  columns: string[];
};

export type QueryResult = {
  columns: string[];
  rows: (string | null)[][];
  row_count: number;
  affected: number;
  execution_ms: number;
  has_result_set: boolean;
};

/** Các loại DB có thể chọn trong form (chỉ `postgres` chạy được ở phiên bản này). */
export type DbTypeOption = {
  value: string;
  label: string;
  defaultPort: number;
  supported: boolean;
};

export const DB_TYPE_OPTIONS: DbTypeOption[] = [
  { value: "postgres", label: "PostgreSQL", defaultPort: 5432, supported: true },
  { value: "mysql", label: "MySQL", defaultPort: 3306, supported: false },
  { value: "sqlite", label: "SQLite", defaultPort: 0, supported: false },
  { value: "sqlserver", label: "SQL Server", defaultPort: 1433, supported: false },
];
