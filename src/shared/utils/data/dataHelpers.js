/**
 * Data Helper Utilities
 * Shared utilities for data manipulation and extraction
 */

/**
 * Get distinct values from data (client-side)
 *
 * @param {Object} data - The data object containing rows and columns
 * @param {string|Object} column - The column name or column object
 * @returns {Array<string>} Sorted array of distinct values
 */
export function getDistinctValues(data, column) {
  if (!data?.rows || !data?.columns || !column) return [];

  // Find column index
  let columnIndex = -1;
  const columnName = typeof column === "object" ? column?.name : column;

  for (let i = 0; i < data.columns.length; i++) {
    const col = data.columns[i];
    if (!col) continue;
    const colName = typeof col === "object" ? col.name : col;
    if (colName === columnName) {
      columnIndex = i;
      break;
    }
  }

  if (columnIndex === -1) return [];

  const values = new Set();
  for (const row of data.rows) {
    // Handle both array and object row formats
    const value = Array.isArray(row) ? row[columnIndex] : row[columnName];
    if (value !== null && value !== undefined) {
      values.add(String(value));
    }
  }

  return Array.from(values).sort();
}
