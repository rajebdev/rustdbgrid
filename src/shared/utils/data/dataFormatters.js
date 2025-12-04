/**
 * Data Formatters Utility
 * Handles value formatting and column type detection
 */

/**
 * Format value for display
 */
export function formatValue(value) {
  if (value === null || value === undefined) {
    return "NULL";
  }
  if (Array.isArray(value)) {
    if (value.length === 0) {
      return "[]";
    }
    const formatted = value.map((item) => {
      if (item === null) return "NULL";
      if (typeof item === "object") return JSON.stringify(item);
      return String(item);
    });
    return `[${formatted.join(", ")}]`;
  }
  if (typeof value === "object") {
    return JSON.stringify(value);
  }
  return value.toString();
}

/**
 * Detect if column is numeric by sampling first 10 rows
 */
export function isNumericColumn(data, column) {
  if (!data?.rows || data.rows.length === 0) return false;

  const sampleSize = Math.min(10, data.rows.length);
  let numericCount = 0;

  for (let i = 0; i < sampleSize; i++) {
    const value = data.rows[i][column];
    if (value === null || value === undefined) continue;
    if (typeof value === "number") {
      numericCount++;
    }
  }

  return numericCount / sampleSize > 0.7;
}

/**
 * Stringify row with consistent key order based on columns
 */
export function stringifyRowWithOrder(row, columns) {
  const orderedRow = {};
  columns.forEach((column, index) => {
    // Handle both array and object row formats
    const columnInfo = typeof column === "object" ? column.name : column;
    const value = Array.isArray(row) ? row[index] : row[columnInfo];
    orderedRow[columnInfo] = value;
  });
  return JSON.stringify(orderedRow, null, 2);
}

/**
 * Format array cell for display
 */
export function formatArrayCell(value, maxLength = 50) {
  if (!Array.isArray(value)) return formatValue(value);

  if (value.length === 0) return "[]";

  const formatted = value
    .slice(0, 3)
    .map((item) => {
      if (item === null) return "NULL";
      return String(item).substring(0, 20);
    })
    .join(", ");

  return value.length > 3 ? `[${formatted}, ...]` : `[${formatted}]`;
}
