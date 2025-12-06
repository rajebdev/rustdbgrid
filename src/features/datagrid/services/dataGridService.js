/**
 * DataGrid Service
 * Handles all data loading, pagination, and filtering operations
 */

import {
  getFilterValues,
  getDistinctValues,
  loadTableDataRaw,
} from "../../../core/integrations/tauri";
import { DatabaseType } from "../../../core/config/databaseTypes";

/**
 * Convert column filters object to array format for backend
 * @param {Object} columnFilters - Object with column names as keys and filter values
 * @returns {Array} Array of filter objects with column, operator, and value
 */
function convertColumnFiltersToArray(columnFilters) {
  const filters = [];
  if (!columnFilters || typeof columnFilters !== "object") {
    return filters;
  }
  for (const [column, value] of Object.entries(columnFilters)) {
    if (Array.isArray(value) && value.length > 0) {
      // Array filter - use "in" operator
      filters.push({
        column,
        operator: "in",
        value: value,
      });
    } else if (typeof value === "string" && value.trim() !== "") {
      // Text filter - use "like" operator
      filters.push({
        column,
        operator: "like",
        value: `%${value}%`,
      });
    }
  }
  return filters;
}

/**
 * Convert sort stack to orderBy array format for backend
 * @param {Array} sortStack - Array of sort objects [{ column, direction, priority }]
 * @returns {Array} Array of orderBy objects sorted by priority
 */
function convertSortStackToOrderBy(sortStack) {
  if (!sortStack || sortStack.length === 0) {
    return [];
  }

  // Sort by priority and map to orderBy format
  return sortStack
    .sort((a, b) => a.priority - b.priority)
    .map((sort) => ({
      column: sort.column,
      direction: sort.direction.toLowerCase(),
    }));
}

/**
 * Load table data with filters and sorting (offset always 0)
 * Used for initial load from sidebar table selection
 */
export async function loadTableInitial(
  connection,
  tableName,
  databaseName,
  schemaName,
  columnFilters,
  sortStack,
  limit = 200
) {
  if (!connection || !tableName) {
    return null;
  }

  try {
    const filters = convertColumnFiltersToArray(columnFilters);
    const orderBy = convertSortStackToOrderBy(sortStack);

    console.log(
      `[loadTableInitial] Loading ${tableName} with orderBy:`,
      orderBy
    );

    const result = await loadTableDataRaw(
      connection.id,
      connection.db_type,
      tableName,
      {
        database: databaseName || null,
        schema: schemaName || null,
        limit: limit,
        offset: 0,
        filters,
        orderBy,
      }
    );

    console.log(`[loadTableInitial] Result received:`, {
      columns: result?.columns?.length || 0,
      rows: result?.rows?.length || 0,
      final_query: result?.final_query,
      first_row: result?.rows?.[0],
      last_row: result?.rows?.[result?.rows?.length - 1],
    });

    return result;
  } catch (error) {
    console.error("❌ Failed to load table data:", error);
    throw error;
  }
}

/**
 * Load custom query data with filters and sorting (offset always 0)
 * Used for initial load from SQL editor query execution
 * Handles Apache Ignite special case with client-side filtering/sorting
 */
export async function loadQueryInitial(
  connection,
  executedQuery,
  tableName,
  databaseName,
  columnFilters,
  sortStack,
  limit = 200
) {
  if (!executedQuery || executedQuery.trim() === "") {
    return null;
  }

  if (!connection) {
    return null;
  }

  try {
    // Convert columnFilters to the format expected by backend
    const filters = {};
    if (columnFilters && typeof columnFilters === "object") {
      for (const [col, value] of Object.entries(columnFilters)) {
        if (Array.isArray(value) && value.length > 0) {
          filters[col] = value;
        } else if (typeof value === "string" && value.trim() !== "") {
          filters[col] = value;
        }
      }
    }

    const dbType = connection?.db_type || DatabaseType.MYSQL;
    const isIgnite = dbType === DatabaseType.IGNITE;

    let result;

    // For Apache Ignite, ALWAYS use SCAN - never use SQL query
    if (isIgnite) {
      let cacheName = databaseName || tableName || "";

      if (!cacheName) {
        const scanMatch = executedQuery.match(/^SCAN\s+(\S+)/i);
        cacheName = scanMatch ? scanMatch[1] : "";
      }

      if (cacheName) {
        result = await loadTableDataRaw({
          connection_id: connection.id,
          query: {
            db_type: DatabaseType.IGNITE,
            database: cacheName,
            schema: null,
            table: "",
            limit: limit,
            offset: 0,
            filters: null,
            order_by: null,
          },
        });

        // Apply client-side filtering for Ignite if filters are set
        if (
          Object.keys(filters).length > 0 &&
          result?.rows &&
          result?.columns
        ) {
          const filteredRows = result.rows.filter((row) => {
            return Object.entries(filters).every(([column, filterValue]) => {
              // Find column index
              let columnIndex = -1;
              for (let i = 0; i < result.columns.length; i++) {
                const col = result.columns[i];
                if (!col) continue;
                const colName = typeof col === "object" ? col.name : col;
                if (colName === column) {
                  columnIndex = i;
                  break;
                }
              }

              // Handle both array and object row formats
              const cellValue = Array.isArray(row)
                ? row[columnIndex]
                : row[column];
              const cellStr =
                cellValue === null || cellValue === undefined
                  ? "NULL"
                  : String(cellValue);

              if (Array.isArray(filterValue)) {
                return filterValue.some((fv) => {
                  if (fv === "NULL")
                    return cellValue === null || cellValue === undefined;
                  return cellStr === fv;
                });
              } else if (
                typeof filterValue === "string" &&
                filterValue.trim() !== ""
              ) {
                return cellStr
                  .toLowerCase()
                  .includes(filterValue.toLowerCase());
              }
              return true;
            });
          });
          result = { ...result, rows: filteredRows };
        }

        // Apply client-side sorting for Ignite if sort is set
        if (sortColumn && result?.rows && result?.columns) {
          // Find sort column index
          let sortColumnIndex = -1;
          for (let i = 0; i < result.columns.length; i++) {
            const col = result.columns[i];
            if (!col) continue;
            const colName = typeof col === "object" ? col.name : col;
            if (colName === sortColumn) {
              sortColumnIndex = i;
              break;
            }
          }

          // Apply client-side sorting for Ignite using sortStack
          result.rows.sort((a, b) => {
            for (const sort of sortStack) {
              const sortColumnName = sort.column;
              let sortColumnIndex = -1;
              for (let i = 0; i < result.columns.length; i++) {
                const col = result.columns[i];
                if (!col) continue;
                const colName = typeof col === "object" ? col.name : col;
                if (colName === sortColumnName) {
                  sortColumnIndex = i;
                  break;
                }
              }

              if (sortColumnIndex < 0) continue;

              const direction =
                sort.direction.toUpperCase() === "DESC" ? -1 : 1;
              const aVal = Array.isArray(a)
                ? a[sortColumnIndex]
                : a[sortColumnName];
              const bVal = Array.isArray(b)
                ? b[sortColumnIndex]
                : b[sortColumnName];

              if (aVal === null || aVal === undefined) {
                if (bVal === null || bVal === undefined) continue;
                return direction;
              }
              if (bVal === null || bVal === undefined) return -direction;

              let comparison = 0;
              if (typeof aVal === "number" && typeof bVal === "number") {
                comparison = (aVal - bVal) * direction;
              } else {
                comparison =
                  String(aVal).localeCompare(String(bVal)) * direction;
              }

              if (comparison !== 0) return comparison;
            }
            return 0;
          });
        }
      } else {
        throw new Error(
          "Cannot determine cache name for Ignite. Please specify cache name."
        );
      }
    } else {
      // Convert columnFilters to new filter format
      const filterArray = convertColumnFiltersToArray(columnFilters);
      const orderBy = convertSortStackToOrderBy(sortStack);

      // Use loadTableDataRaw with subquery wrapper
      result = await loadTableDataRaw(
        connection.id,
        connection.db_type,
        `RustDBGridQuery(${executedQuery})`,
        {
          limit: limit,
          offset: 0,
          filters: filterArray,
          orderBy,
        }
      );
    }

    return result;
  } catch (error) {
    console.error("❌ Failed to reload data with filters:", error);
    throw error;
  }
}

/**
 * Append more table data for pagination (offset incremental)
 * Continues loading from existing table with filter/sort applied
 */
export async function appendTableData(
  connection,
  tableName,
  databaseName,
  schemaName,
  columnFilters,
  sortStack,
  currentOffset,
  paginateLimit = 200
) {
  if (!connection || !tableName) {
    return null;
  }

  try {
    const filters = convertColumnFiltersToArray(columnFilters);
    const orderBy = convertSortStackToOrderBy(sortStack);

    console.log(
      `[appendTableData] Appending ${tableName} at offset ${currentOffset} with limit ${paginateLimit} and orderBy:`,
      orderBy
    );

    const result = await loadTableDataRaw(
      connection.id,
      connection.db_type,
      tableName,
      {
        database: databaseName || null,
        schema: schemaName || null,
        limit: paginateLimit,
        offset: currentOffset,
        filters,
        orderBy,
      }
    );

    console.log(`[appendTableData] Result:`, {
      rows: result?.rows?.length || 0,
      first_row: result?.rows?.[0],
    });

    return result;
  } catch (error) {
    console.error("❌ Failed to append table data:", error);
    throw error;
  }
}

/**
 * Append more query data for pagination (offset incremental)
 * Continues loading from existing custom query with filter/sort applied
 */
export async function appendQueryData(
  connection,
  executedQuery,
  tableName,
  databaseName,
  columnFilters,
  sortStack,
  currentOffset,
  paginateLimit = 200
) {
  if (!connection || !executedQuery || executedQuery.trim() === "") {
    return null;
  }

  try {
    const filters = convertColumnFiltersToArray(columnFilters);
    const orderBy = convertSortStackToOrderBy(sortStack);

    console.log(
      `[appendQueryData] Appending query at offset ${currentOffset} with limit ${paginateLimit} and orderBy:`,
      orderBy
    );

    const result = await loadTableDataRaw(
      connection.id,
      connection.db_type,
      `RustDBGridQuery(${executedQuery})`,
      {
        limit: paginateLimit,
        offset: currentOffset,
        filters,
        orderBy,
      }
    );

    console.log(`[appendQueryData] Result:`, {
      rows: result?.rows?.length || 0,
      first_row: result?.rows?.[0],
    });

    return result;
  } catch (error) {
    console.error("❌ Failed to append query data:", error);
    throw error;
  }
}

/**
 * Load filter values from server for column
 */
export async function loadFilterValuesFromServer(
  connection,
  tableName,
  databaseName,
  schemaName,
  column,
  searchTerm = null
) {
  if (!connection || !tableName || !column) {
    return [];
  }

  try {
    // Use structured request approach (similar to loadTableDataRaw)
    const result = await getDistinctValues(
      connection.id,
      connection.db_type,
      tableName,
      column,
      {
        database: databaseName,
        schema: schemaName,
        searchTerm: searchTerm,
        limit: 1000,
      }
    );

    console.log("📥 Loaded distinct values:", {
      column,
      count: result?.total_count,
      executionTime: result?.execution_time,
    });

    return result?.values || [];
  } catch (error) {
    console.error("❌ Failed to load filter values:", error);
    return [];
  }
}
