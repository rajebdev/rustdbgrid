/**
 * Format bytes to human-readable format
 */
export function formatBytes(bytes) {
  if (bytes === null || bytes === undefined) return "N/A";
  if (bytes === 0) return "0";
  const k = 1024;
  const sizes = ["B", "K", "M", "G", "T"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + "" + sizes[i];
}

/**
 * Format number with locale string
 */
export function formatNumber(num) {
  if (num === null || num === undefined) return "N/A";
  return num.toLocaleString();
}

/**
 * Get data type display from column
 */
export function getDataTypeDisplay(column) {
  return column.data_type || "unknown";
}

/**
 * Get column key indicator
 */
export function getColumnKey(column) {
  if (column.is_primary_key) return "PRI";
  return "";
}
