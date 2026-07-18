import { safeInvoke } from "./_base";
import type {
  DbTable,
  QueryResult,
  SaveSqlConnectionRequest,
  SqlConnection,
} from "@/_/types/sql-editor";

export function sqlListConnections() {
  return safeInvoke<SqlConnection[]>("sql_list_connections");
}

export function sqlSaveConnection(request: SaveSqlConnectionRequest) {
  return safeInvoke<SqlConnection>("sql_save_connection", { request });
}

export function sqlDeleteConnection(id: number) {
  return safeInvoke<void>("sql_delete_connection", { id });
}

export function sqlTestConnection(request: SaveSqlConnectionRequest) {
  return safeInvoke<void>("sql_test_connection", { request });
}

export function sqlGetSchema(connectionId: number) {
  return safeInvoke<DbTable[]>("sql_get_schema", { connectionId });
}

export function sqlRunQuery(connectionId: number, sql: string) {
  return safeInvoke<QueryResult>("sql_run_query", { connectionId, sql });
}
