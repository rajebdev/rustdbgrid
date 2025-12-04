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
 * Generic load handler factory
 * Creates a load handler with abort controller and loading state management
 */
function createLoadHandler(config) {
  const {
    getOperationKey,
    getLoadingKey,
    setLoadingState,
    loadData,
    processResult,
    errorMessage = "Failed to load data",
  } = config;

  return async (...args) => {
    const opKey = getOperationKey(...args);
    const loadingKey = getLoadingKey ? getLoadingKey(...args) : null;

    // Cancel previous operation if exists
    cancelOperation(opKey);

    const controller = new AbortController();
    abortControllers.set(opKey, controller);

    if (setLoadingState && loadingKey) {
      setLoadingState(loadingKey, true);
    }

    try {
      const result = await loadData(...args);

      if (controller.signal.aborted) return null;

      await processResult(result, ...args);

      return result;
    } catch (error) {
      if (controller.signal.aborted) return null;
      console.error(errorMessage, error);
      throw error;
    } finally {
      if (setLoadingState && loadingKey) {
        setLoadingState(loadingKey, false);
      }
      abortControllers.delete(opKey);
    }
  };
}

/**
 * Load databases for a connection
 */
export const loadConnectionDatabases = createLoadHandler({
  getOperationKey: (connId) => `conn-${connId}`,
  getLoadingKey: (connId) => connId,
  setLoadingState: (key, loading) =>
    sidebarStore.setConnectionLoading(key, loading),
  loadData: async (connId) => {
    const result = await getDatabaseObject(connId, "database_list");
    return result.databases || [];
  },
  processResult: (databases, connId) => {
    sidebarStore.toggleConnection(connId, { databases });
    sidebarStore.setConnectionConnected(connId, true);
  },
  errorMessage: "Failed to load databases:",
});

/**
 * Load database info (tables, views, etc.)
 */
export const loadDatabaseInfo = createLoadHandler({
  getOperationKey: (connId, dbName) => `db-${connId}-${dbName}`,
  getLoadingKey: (connId, dbName) => `${connId}-${dbName}`,
  setLoadingState: (key, loading) =>
    sidebarStore.setDatabaseLoading(key, loading),
  loadData: async (connId, dbName, dbType) => {
    if (dbType === DatabaseType.POSTGRESQL) {
      return getDatabaseObject(connId, "schema_list", dbName);
    } else {
      return getDatabaseObject(connId, "database_info", dbName);
    }
  },
  processResult: (result, connId, dbName, dbType) => {
    const dbKey = `${connId}-${dbName}`;

    if (dbType === DatabaseType.POSTGRESQL) {
      sidebarStore.setCachedData(dbKey, { schemas: result.schemas || [] });
    } else {
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
  },
  errorMessage: "Failed to load database:",
});

/**
 * Load schema info (tables, views, etc.)
 */
export const loadSchemaInfo = createLoadHandler({
  getOperationKey: (connId, dbName, schemaName) =>
    `schema-${connId}-${dbName}-${schemaName}`,
  getLoadingKey: (connId, dbName, schemaName) =>
    `${connId}-${dbName}-${schemaName}`,
  setLoadingState: (key, loading) =>
    sidebarStore.setSchemaLoading(key, loading),
  loadData: async (connId, dbName, schemaName) => {
    return getDatabaseObject(connId, "schema_info", dbName, schemaName);
  },
  processResult: (result, connId, dbName, schemaName) => {
    const schemaKey = `${connId}-${dbName}-${schemaName}`;

    sidebarStore.setCachedData(schemaKey, {
      tables: result.tables || [],
      views: result.views || [],
      indexes: result.indexes || [],
      procedures: result.procedures || [],
      triggers: result.triggers || [],
    });

    sidebarStore.toggleSchema(schemaKey, true);
  },
  errorMessage: "Failed to load schema:",
});

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
 * @param {string} connId - Connection ID
 * @param {boolean} preserveExpandState - If true, don't collapse the connection
 */
export async function disconnectDatabase(connId, preserveExpandState = false) {
  const opKey = `disconnect-${connId}`;

  cancelOperation(opKey);

  const controller = new AbortController();
  abortControllers.set(opKey, controller);

  sidebarStore.setConnectionLoading(connId, true);

  try {
    await disconnectFromDatabase(connId);

    if (controller.signal.aborted) return;

    await syncConnectedStatus();

    // Collapse connection after disconnect (unless we want to preserve state)
    if (!preserveExpandState) {
      sidebarStore.toggleConnection(connId, null);
    }
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

  // Save expanded state before disconnect
  let wasExpanded = false;
  const unsubscribe = sidebarStore.subscribe((state) => {
    wasExpanded = !!state.expandedConnections[connId];
  });
  unsubscribe();

  try {
    // Disconnect from backend but don't collapse UI
    await disconnectFromDatabase(connId);

    if (controller.signal.aborted) return;

    await connectToDatabase(connId);

    if (controller.signal.aborted) return;

    await syncConnectedStatus();

    // Reload data if was expanded
    if (wasExpanded) {
      await loadConnectionDatabases(connId);
    }
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
 * Generic refresh handler for tables and views
 * Refreshes a specific object type by reloading database info
 */
export async function refreshDatabaseObject(connId, dbName, objectType) {
  const dbKey = `${connId}-${dbName}`;

  try {
    const response = await getDatabaseObject(connId, "database_info", dbName);

    if (response && response[objectType]) {
      sidebarStore.updateCachedData(dbKey, (data) => ({
        ...data,
        [objectType]: response[objectType],
      }));
    }
  } catch (error) {
    console.error(`Failed to refresh ${objectType}:`, error);
    throw error;
  }
}

/**
 * Helper to get group key
 */
export function getGroupKey(connId, dbName, type, schemaName = null) {
  return schemaName
    ? `${connId}-${dbName}-${schemaName}-${type}`
    : `${connId}-${dbName}-${type}`;
}
