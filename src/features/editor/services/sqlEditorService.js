import {
  getDatabaseObject,
  getPropertiesObject,
  saveAutoQuery,
  loadAutoQuery,
} from "../utils/tauri";

/**
 * Load databases for SQL editor
 */
export async function loadDatabasesForEditor(connId) {
  if (!connId) return [];

  try {
    const result = await getDatabaseObject(connId, "database_list");
    return result.databases || [];
  } catch (error) {
    console.error("Failed to load databases:", error);
    return [];
  }
}

/**
 * Lazy load tables for a specific database
 */
export async function loadTablesForDatabase(connId, dbName, cache = {}) {
  if (!connId || !dbName) return [];

  // Return from cache if available
  if (cache[dbName]) {
    return cache[dbName].tables;
  }

  try {
    const result = await getDatabaseObject(connId, "database_info", dbName);
    const tables = result.tables || [];

    // Store in cache
    cache[dbName] = { tables, schema: {} };
    return tables;
  } catch (error) {
    console.error(`Failed to load tables for ${dbName}:`, error);
    return [];
  }
}

/**
 * Lazy load columns for a specific table
 */
export async function loadColumnsForTable(
  connId,
  dbName,
  tableName,
  cache = {}
) {
  if (!connId || !dbName || !tableName) return [];

  // Check if already loaded in cache
  if (cache[dbName]?.schema?.[tableName]) {
    return cache[dbName].schema[tableName];
  }

  try {
    const tableSchema = await getPropertiesObject(
      connId,
      "schema",
      dbName,
      tableName
    );
    const columns = tableSchema.columns.map((col) => col.name);

    // Initialize cache structure if needed
    if (!cache[dbName]) {
      cache[dbName] = { tables: [], schema: {} };
    }
    if (!cache[dbName].schema) {
      cache[dbName].schema = {};
    }
    cache[dbName].schema[tableName] = columns;

    return columns;
  } catch (error) {
    console.error(`Failed to load columns for ${dbName}.${tableName}:`, error);
    return [];
  }
}

/**
 * Load auto-saved query from backend
 */
export async function loadAutoSavedQuery(tabId) {
  try {
    const autoSaved = await loadAutoQuery();
    if (autoSaved && autoSaved.tab_id === tabId) {
      return autoSaved.query;
    }
    return null;
  } catch (error) {
    console.error("Failed to load auto-saved query:", error);
    return null;
  }
}

/**
 * Save query with auto-save
 */
export async function saveQueryAuto(tabId, queryText, connId, dbName) {
  if (!queryText.trim()) return;

  try {
    await saveAutoQuery(tabId, queryText, connId, dbName);
  } catch (error) {
    console.error("Failed to auto-save query:", error);
    throw error;
  }
}

/**
 * Generate table alias
 */
export function generateTableAlias(tableName, existingAliases = new Set()) {
  // Convert to lowercase and get first letter of each word
  const words = tableName.toLowerCase().split(/[_-]/);
  let alias = "";

  if (words.length === 1) {
    // Single word: take first letter
    alias = words[0][0];
  } else {
    // Multiple words: take first letter of each
    alias = words.map((w) => w[0]).join("");
  }

  // Check for duplicates and append number if needed
  let finalAlias = alias;
  let counter = 1;

  while (existingAliases.has(finalAlias)) {
    finalAlias = alias + counter;
    counter++;
  }

  return finalAlias;
}
