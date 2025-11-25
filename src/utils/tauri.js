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
  return await invoke("execute_query_with_filters", {
    config,
    base_query: baseQuery,
    filters,
    sort_column: sortColumn,
    sort_direction: sortDirection,
    limit,
  });
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
