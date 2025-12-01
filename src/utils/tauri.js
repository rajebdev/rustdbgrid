import { invoke } from "@tauri-apps/api/core";

export async function testConnection(config) {
  return await invoke("test_connection", { config });
}

export async function saveConnection(config) {
  return await invoke("save_connection", { config });
}

export async function getConnections() {
  return await invoke("get_connections");
}

export async function deleteConnection(id) {
  return await invoke("delete_connection", { id });
}

export async function updateConnection(config) {
  return await invoke("update_connection", { config });
}

export async function connectToDatabase(config) {
  return await invoke("connect_to_database", { config });
}

export async function disconnectFromDatabase(connectionId) {
  return await invoke("disconnect_from_database", { connectionId });
}

export async function isDatabaseConnected(connectionId) {
  return await invoke("is_database_connected", { connectionId });
}

export async function getConnectedDatabases() {
  return await invoke("get_connected_databases");
}

export async function executeQuery(config, query) {
  return await invoke("execute_query", { config, query });
}

export async function executeQueryWithFilters(
  config,
  baseQuery,
  filters = null,
  sortColumn = null,
  sortDirection = null,
  limit = null,
  offset = null
) {
  if (!baseQuery || baseQuery.trim() === "") {
    throw new Error("baseQuery is required and cannot be empty");
  }

  if (!config) {
    throw new Error("config is required");
  }

  console.log("📤 executeQueryWithFilters called with:", {
    config: config
      ? {
          id: config.id,
          name: config.name,
          db_type: config.db_type,
        }
      : "NULL",
    baseQuery: baseQuery.substring(0, 100),
    filters,
    sortColumn,
    sortDirection,
    limit,
    offset,
  });

  const payload = {
    config,
    baseQuery: baseQuery, // Try camelCase instead of snake_case
    filters,
    sortColumn: sortColumn,
    sortDirection: sortDirection,
    limit,
    offset,
  };

  console.log("📦 Full payload:", JSON.stringify(payload, null, 2));

  return await invoke("execute_query_with_filters", payload);
}

export async function getFilterValues(
  config,
  query,
  column,
  searchQuery = null,
  limit = 1000
) {
  return await invoke("get_filter_values", {
    config,
    query,
    column,
    search_query: searchQuery,
    limit,
  });
}

export async function getDatabases(config) {
  return await invoke("get_databases", { config });
}

export async function getTables(config, database) {
  return await invoke("get_tables", { config, database });
}

export async function getViews(config, database, schema = null) {
  return await invoke("get_views", { config, database, schema });
}

export async function getIndexes(config, database, schema = null) {
  return await invoke("get_indexes", { config, database, schema });
}

export async function getProcedures(config, database, schema = null) {
  return await invoke("get_procedures", { config, database, schema });
}

export async function getTriggers(config, database, schema = null) {
  return await invoke("get_triggers", { config, database, schema });
}

export async function getEvents(config, database, schema = null) {
  return await invoke("get_events", { config, database, schema });
}

export async function getTableSchema(config, database, table) {
  return await invoke("get_table_schema", { config, database, table });
}

export async function getTableRelationships(config, database, table) {
  return await invoke("get_table_relationships", { config, database, table });
}

export async function getTableData(
  config,
  database,
  table,
  limit = 100,
  offset = 0
) {
  return await invoke("get_table_data", {
    config,
    database,
    table,
    limit,
    offset,
  });
}

export async function getStorageInfo() {
  return await invoke("get_storage_info");
}

// Settings commands
export async function getSettings() {
  return await invoke("get_settings");
}

export async function saveSettings(settings) {
  return await invoke("save_settings", { settings });
}

export async function updateSetting(key, value) {
  return await invoke("update_setting", { key, value });
}

export async function getTheme() {
  return await invoke("get_theme");
}

export async function setTheme(theme) {
  return await invoke("set_theme", { theme });
}

// Query management functions
export async function saveQuery(
  title,
  content,
  description = "",
  connectionId = null,
  databaseName = null
) {
  return await invoke("save_query", {
    title,
    content,
    description,
    connection_id: connectionId,
    database_name: databaseName,
  });
}

export async function loadQueries() {
  return await invoke("load_queries");
}

export async function deleteQuery(queryId) {
  return await invoke("delete_query", { query_id: queryId });
}

export async function saveAutoQuery(
  tabId,
  query,
  connectionId = null,
  databaseName = null
) {
  return await invoke("save_auto_query", {
    tab_id: tabId,
    query,
    connection_id: connectionId,
    database_name: databaseName,
  });
}

export async function loadAutoQuery() {
  return await invoke("load_auto_query");
}

export async function getNextQueryNumber() {
  return await invoke("get_next_query_number");
}
