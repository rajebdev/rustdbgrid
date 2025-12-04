/**
 * DataGrid Service
 * Handles all data loading, pagination, and filtering operations
 */

import { getFilterValues, loadTableData } from "../utils/tauri";
import { DatabaseType } from "../utils/databaseTypes";

/**
 * Convert column filters object to array format for backend
 * @param {Object} columnFilters - Object with column names as keys and filter values
 * @returns {Array} Array of filter objects with column, operator, and value
 */
function convertColumnFiltersToArray(columnFilters) {
  const filters = [];
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
 * Convert sort column and direction to orderBy array format for backend
 * @param {string} sortColumn - Column name to sort by
 * @param {string} sortDirection - Sort direction (ASC/DESC)
 * @returns {Array} Array with single orderBy object, or empty array if no sort
 */
function convertSortToOrderBy(sortColumn, sortDirection) {
  return sortColumn
    ? [
        {
          column: sortColumn,
          direction: sortDirection.toLowerCase(),
        },
      ]
    : [];
}

/**
 * Load table data with filters, sorting, and pagination
 */
export async function loadTableDataWithFilters(
  connection,
  tableName,
  databaseName,
  schemaName,
  columnFilters,
  sortColumn,
  sortDirection
) {
  if (!connection || !tableName) {
    return null;
  }

  try {
    const filters = convertColumnFiltersToArray(columnFilters);
    const orderBy = convertSortToOrderBy(sortColumn, sortDirection);

    const result = await loadTableData(
      connection.id,
      connection.db_type,
      tableName,
      {
        database: databaseName || null,
        schema: schemaName || null,
        limit: 200,
        offset: 0,
        filters,
        orderBy,
      }
    );

    return result;
  } catch (error) {
    console.error("❌ Failed to load table data:", error);
    throw error;
  }
}

/**
 * Reload data with filters and sorting (for SQL query results)
 */
export async function reloadDataWithFilters(
  connection,
  executedQuery,
  tableName,
  databaseName,
  columnFilters,
  sortColumn,
  sortDirection
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
    for (const [col, value] of Object.entries(columnFilters)) {
      if (Array.isArray(value) && value.length > 0) {
        filters[col] = value;
      } else if (typeof value === "string" && value.trim() !== "") {
        filters[col] = value;
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
        result = await loadTableData({
          connection_id: connection.id,
          query: {
            db_type: DatabaseType.IGNITE,
            database: cacheName,
            schema: null,
            table: "",
            limit: 200,
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

          const direction = sortDirection.toUpperCase() === "DESC" ? -1 : 1;
          result.rows.sort((a, b) => {
            // Handle both array and object row formats
            const aVal = Array.isArray(a) ? a[sortColumnIndex] : a[sortColumn];
            const bVal = Array.isArray(b) ? b[sortColumnIndex] : b[sortColumn];
            if (aVal === null || aVal === undefined) return direction;
            if (bVal === null || bVal === undefined) return -direction;
            if (typeof aVal === "number" && typeof bVal === "number") {
              return (aVal - bVal) * direction;
            }
            return String(aVal).localeCompare(String(bVal)) * direction;
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
      const orderBy = convertSortToOrderBy(sortColumn, sortDirection);

      // Use loadTableData with subquery wrapper
      result = await loadTableData(
        connection.id,
        connection.db_type,
        `RustDBGridQuery(${executedQuery})`,
        {
          limit: 200,
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
 * Load more data for pagination
 */
export async function loadMoreData(
  connection,
  tableName,
  databaseName,
  schemaName,
  columnFilters,
  sortColumn,
  sortDirection,
  currentOffset,
  executedQuery = null
) {
  if (!connection) {
    return null;
  }

  // Must have either table name or executed query
  if (!tableName && (!executedQuery || executedQuery.trim() === "")) {
    return null;
  }

  try {
    const filters = convertColumnFiltersToArray(columnFilters);
    const orderBy = convertSortToOrderBy(sortColumn, sortDirection);

    let result;

    // If executedQuery exists, wrap it with RustDBGridQuery for custom queries
    if (executedQuery && executedQuery.trim() !== "") {
      result = await loadTableData(
        connection.id,
        connection.db_type,
        `RustDBGridQuery(${executedQuery})`,
        {
          limit: 200,
          offset: currentOffset,
          filters,
          orderBy,
        }
      );
    } else {
      // Normal table query
      result = await loadTableData(
        connection.id,
        connection.db_type,
        tableName,
        {
          database: databaseName || null,
          schema: schemaName || null,
          limit: 200,
          offset: currentOffset,
          filters,
          orderBy,
        }
      );
    }

    return result;
  } catch (error) {
    console.error("❌ Failed to load more data:", error);
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
    // Build a simple SELECT DISTINCT query for the table
    const query = schemaName
      ? `SELECT DISTINCT ${column} FROM ${schemaName}.${tableName}`
      : databaseName
      ? `SELECT DISTINCT ${column} FROM ${databaseName}.${tableName}`
      : `SELECT DISTINCT ${column} FROM ${tableName}`;

    const result = await getFilterValues(
      connection.id,
      query,
      column,
      searchTerm,
      1000
    );

    return result?.values || [];
  } catch (error) {
    console.error("❌ Failed to load filter values:", error);
    return [];
  }
}
