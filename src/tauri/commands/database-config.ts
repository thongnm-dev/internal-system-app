import { safeInvoke } from "./_base";
import type { DatabaseConfig, DatabaseStatus, SaveDatabaseConfigRequest } from "@/_/types/database-config";

export function checkDatabaseStatus() {
  return safeInvoke<DatabaseStatus>("check_database_status");
}

export function getDatabaseConfig() {
  return safeInvoke<DatabaseConfig | null>("get_database_config");
}

export function testDatabaseConfig(request: SaveDatabaseConfigRequest) {
  return safeInvoke<void>("test_database_config", { request });
}

export function saveDatabaseConfig(request: SaveDatabaseConfigRequest) {
  return safeInvoke<DatabaseStatus>("save_database_config", { request });
}
