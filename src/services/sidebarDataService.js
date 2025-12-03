import {
  getDatabaseObject,
  getConnectedDatabases,
  connectToDatabase,
  disconnectFromDatabase,
} from "../utils/tauri";
import { sidebarStore } from "../stores/sidebar";
import { DatabaseType } from "../utils/databaseTypes";

// Active abort controllers for cancellable operations
const abortControllers = new Map();

/**
 * Cancel an ongoing operation
 */
export function cancelOperation(key) {
  const controller = abortControllers.get(key);
  if (controller) {
    controller.abort();
    abortControllers.delete(key);
  }
}

/**
 * Cancel all ongoing operations
 */
export function cancelAllOperations() {
  abortControllers.forEach((controller) => controller.abort());
  abortControllers.clear();
}

/**
 * Load databases for a connection
 */
export async function loadConnectionDatabases(connId) {
  const opKey = `conn-${connId}`;

  // Cancel previous operation if exists
  cancelOperation(opKey);

  const controller = new AbortController();
  abortControllers.set(opKey, controller);

  sidebarStore.setConnectionLoading(connId, true);

  try {
    const result = await getDatabaseObject(connId, "database_list");

    if (controller.signal.aborted) return null;

    const databases = result.databases || [];
    sidebarStore.toggleConnection(connId, { databases });
    sidebarStore.setConnectionConnected(connId, true);

    return databases;
  } catch (error) {
    if (controller.signal.aborted) return null;
    console.error("Failed to load databases:", error);
    throw error;
  } finally {
    sidebarStore.setConnectionLoading(connId, false);
    abortControllers.delete(opKey);
  }
}

/**
 * Load database info (tables, views, etc.)
 */
export async function loadDatabaseInfo(connId, dbName, dbType) {
  const dbKey = `${connId}-${dbName}`;
  const opKey = `db-${dbKey}`;

  cancelOperation(opKey);

  const controller = new AbortController();
  abortControllers.set(opKey, controller);

  sidebarStore.setDatabaseLoading(dbKey, true);

  try {
    let result;

    if (dbType === DatabaseType.POSTGRESQL) {
      // PostgreSQL: Load schemas
      result = await getDatabaseObject(connId, "schema_list", dbName);

      if (controller.signal.aborted) return null;

      sidebarStore.setCachedData(dbKey, { schemas: result.schemas || [] });
    } else {
      // MySQL/MSSQL/Others: Load database info
      result = await getDatabaseObject(connId, "database_info", dbName);

      if (controller.signal.aborted) return null;

      sidebarStore.setCachedData(dbKey, {
        tables: result.tables || [],
        views: result.views || [],
        indexes: result.indexes || [],
        procedures: result.procedures || [],
        triggers: result.triggers || [],
        events: result.events || [],
      });
    }

    sidebarStore.toggleDatabase(dbKey, true);

    return result;
  } catch (error) {
    if (controller.signal.aborted) return null;
    console.error("Failed to load database:", error);
    throw error;
  } finally {
    sidebarStore.setDatabaseLoading(dbKey, false);
    abortControllers.delete(opKey);
  }
}

/**
 * Load schema info (tables, views, etc.)
 */
export async function loadSchemaInfo(connId, dbName, schemaName) {
  const schemaKey = `${connId}-${dbName}-${schemaName}`;
  const opKey = `schema-${schemaKey}`;

  cancelOperation(opKey);

  const controller = new AbortController();
  abortControllers.set(opKey, controller);

  sidebarStore.setSchemaLoading(schemaKey, true);

  try {
    const result = await getDatabaseObject(
      connId,
      "schema_info",
      dbName,
      schemaName
    );

    if (controller.signal.aborted) return null;

    sidebarStore.setCachedData(schemaKey, {
      tables: result.tables || [],
      views: result.views || [],
      indexes: result.indexes || [],
      procedures: result.procedures || [],
      triggers: result.triggers || [],
    });

    sidebarStore.toggleSchema(schemaKey, true);

    return result;
  } catch (error) {
    if (controller.signal.aborted) return null;
    console.error("Failed to load schema:", error);
    throw error;
  } finally {
    sidebarStore.setSchemaLoading(schemaKey, false);
    abortControllers.delete(opKey);
  }
}

/**
 * Sync connected database status
 */
export async function syncConnectedStatus() {
  try {
    const connectedIds = await getConnectedDatabases();
    const connections = connectedIds.reduce((acc, id) => {
      acc[id] = true;
      return acc;
    }, {});
    sidebarStore.setConnectedConnections(connections);
    return connections;
  } catch (error) {
    console.error("Failed to sync connected status:", error);
    throw error;
  }
}

/**
 * Connect to a database
 */
export async function connectDatabase(connId) {
  const opKey = `connect-${connId}`;

  cancelOperation(opKey);

  const controller = new AbortController();
  abortControllers.set(opKey, controller);

  sidebarStore.setConnectionLoading(connId, true);

  try {
    await connectToDatabase(connId);

    if (controller.signal.aborted) return;

    await syncConnectedStatus();
  } catch (error) {
    if (controller.signal.aborted) return;
    console.error("Failed to connect:", error);
    throw error;
  } finally {
    sidebarStore.setConnectionLoading(connId, false);
    abortControllers.delete(opKey);
  }
}

/**
 * Disconnect from a database
 */
export async function disconnectDatabase(connId) {
  const opKey = `disconnect-${connId}`;

  cancelOperation(opKey);

  const controller = new AbortController();
  abortControllers.set(opKey, controller);

  sidebarStore.setConnectionLoading(connId, true);

  try {
    await disconnectFromDatabase(connId);

    if (controller.signal.aborted) return;

    await syncConnectedStatus();

    // Collapse connection after disconnect
    sidebarStore.toggleConnection(connId, null);
  } catch (error) {
    if (controller.signal.aborted) return;
    console.error("Failed to disconnect:", error);
    throw error;
  } finally {
    sidebarStore.setConnectionLoading(connId, false);
    abortControllers.delete(opKey);
  }
}

/**
 * Refresh/reconnect to a database
 */
export async function refreshConnection(connId) {
  const opKey = `refresh-${connId}`;

  cancelOperation(opKey);

  const controller = new AbortController();
  abortControllers.set(opKey, controller);

  sidebarStore.setConnectionLoading(connId, true);

  try {
    await disconnectFromDatabase(connId);

    if (controller.signal.aborted) return;

    await connectToDatabase(connId);

    if (controller.signal.aborted) return;

    await syncConnectedStatus();

    // Reload data
    await loadConnectionDatabases(connId);
  } catch (error) {
    if (controller.signal.aborted) return;
    console.error("Failed to refresh:", error);
    throw error;
  } finally {
    sidebarStore.setConnectionLoading(connId, false);
    abortControllers.delete(opKey);
  }
}

/**
 * Refresh database data
 */
export async function refreshDatabase(connId, dbName, dbType) {
  const dbKey = `${connId}-${dbName}`;

  // Clear cache
  sidebarStore.clearCachedData(dbKey);

  // Reload if expanded
  return loadDatabaseInfo(connId, dbName, dbType);
}

/**
 * Refresh schema data
 */
export async function refreshSchema(connId, dbName, schemaName) {
  const schemaKey = `${connId}-${dbName}-${schemaName}`;

  // Clear cache
  sidebarStore.clearCachedData(schemaKey);

  // Reload if expanded
  return loadSchemaInfo(connId, dbName, schemaName);
}

/**
 * Helper to get group key
 */
export function getGroupKey(connId, dbName, type, schemaName = null) {
  return schemaName
    ? `${connId}-${dbName}-${schemaName}-${type}`
    : `${connId}-${dbName}-${type}`;
}
