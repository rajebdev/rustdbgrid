import { loadTableDataRaw } from "../../../core/integrations/tauri";
import { stripSqlComments } from "../utils/sqlFormatter";
import { get } from "svelte/store";
import { defaultPaginateLimit } from "../../../shared/stores/appSettings";

/**
 * Execute SQL query
 */
export async function executeQuery(config) {
  const { query, connId, dbType, dbName, tabId, createNewTab = false } = config;

  if (!connId || !dbName) {
    throw new Error("Connection and database must be selected");
  }

  if (!query || !query.trim()) {
    throw new Error("Query is empty");
  }

  const cleanedQuery = query.replace(/;+\s*$/, ""); // Remove trailing semicolons
  const startTime = Date.now();

  // Use defaultPaginateLimit from store for consistent pagination
  const limit = get(defaultPaginateLimit);

  // Use loadTableDataRaw with subquery wrapper
  const result = await loadTableDataRaw(
    connId,
    dbType,
    `RustDBGridQuery(${cleanedQuery})`,
    {
      limit,
      offset: 0,
    }
  );

  const executionTime = Date.now() - startTime;

  return {
    result,
    query: cleanedQuery,
    executionTime,
    createNewTab,
  };
}

/**
 * Validate query before execution
 */
export function validateQuery(query, selectedConn, selectedDb) {
  if (!selectedConn || !selectedDb) {
    return {
      valid: false,
      error: "Please select connection and database first",
    };
  }

  if (!query || !query.trim()) {
    return {
      valid: false,
      error: "Query is empty",
    };
  }

  return { valid: true };
}

/**
 * Clean SQL query (remove comments)
 */
export function cleanQuery(query, dbType) {
  const cleaned = stripSqlComments(query, dbType);
  if (!cleaned || !cleaned.trim()) {
    throw new Error("Query is empty after removing comments");
  }
  return cleaned;
}
