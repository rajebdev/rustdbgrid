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

export async function executeQuery(config, query) {
  return await invoke("execute_query", { config, query });
}

export async function executeQueryWithFilters(
  config,
  baseQuery,
  filters = null,
  sortColumn = null,
  sortDirection = null,
  limit = null
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
  });

  const payload = {
    config,
    baseQuery: baseQuery, // Try camelCase instead of snake_case
    filters,
    sortColumn: sortColumn,
    sortDirection: sortDirection,
    limit,
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

export async function getTableSchema(config, database, table) {
  return await invoke("get_table_schema", { config, database, table });
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
