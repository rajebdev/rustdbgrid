/**
 * Shared utility functions for panel components
 */

/**
 * Format timestamp to relative or absolute time
 * @param {string|number} timestamp - Unix timestamp or date string
 * @returns {string} Formatted time string
 */
export function formatTimestamp(timestamp) {
  const date =
    typeof timestamp === "number"
      ? new Date(timestamp * 1000)
      : new Date(timestamp);

  const now = new Date();
  const diffMs = now - date;
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return "just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;

  return date.toLocaleDateString();
}

/**
 * Format time for display (HH:MM:SS)
 * @param {string|number} timestamp - Unix timestamp or date string
 * @returns {string} Time string
 */
export function formatTime(timestamp) {
  const date =
    typeof timestamp === "number"
      ? new Date(timestamp * 1000)
      : new Date(timestamp);
  return date.toLocaleTimeString();
}

/**
 * Format date for display
 * @param {string|number} timestamp - Unix timestamp or date string
 * @returns {string} Date string
 */
export function formatDate(timestamp) {
  const date =
    typeof timestamp === "number"
      ? new Date(timestamp * 1000)
      : new Date(timestamp);
  return date.toLocaleDateString();
}

/**
 * Format execution time in milliseconds
 * @param {number} ms - Time in milliseconds
 * @returns {string} Formatted time (e.g., "123ms" or "1.2s")
 */
export function formatExecutionTime(ms) {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

/**
 * Truncate text with ellipsis
 * @param {string} text - Text to truncate
 * @param {number} maxLength - Maximum length
 * @returns {string} Truncated text
 */
export function truncateText(text, maxLength = 80) {
  if (!text) return "";
  if (text.length <= maxLength) return text;
  return text.substring(0, maxLength) + "...";
}

/**
 * Show confirmation dialog
 * @param {string} message - Confirmation message
 * @returns {boolean} True if confirmed
 */
export function showConfirmation(message) {
  return confirm(message);
}

/**
 * Dispatch event to load query to editor
 * @param {string} query - SQL query to load
 */
export function dispatchLoadQuery(query) {
  window.dispatchEvent(
    new CustomEvent("load-query", {
      detail: { query },
    })
  );
}

/**
 * Filter items by search term across multiple fields
 * @param {Array} items - Items to filter
 * @param {string} searchTerm - Search term
 * @param {string[]} fields - Fields to search in
 * @returns {Array} Filtered items
 */
export function filterItems(items, searchTerm, fields) {
  if (!searchTerm || !searchTerm.trim()) return items;

  const term = searchTerm.toLowerCase();
  return items.filter((item) =>
    fields.some((field) => {
      const value = field.split(".").reduce((obj, key) => obj?.[key], item);
      return value && String(value).toLowerCase().includes(term);
    })
  );
}

/**
 * Export data to JSON file
 * @param {any} data - Data to export
 * @param {string} filename - File name (without extension)
 */
export function exportToJSON(data, filename) {
  try {
    const dataStr = JSON.stringify(data, null, 2);
    const dataBlob = new Blob([dataStr], { type: "application/json" });
    const url = URL.createObjectURL(dataBlob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${filename}_${new Date().toISOString().split("T")[0]}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  } catch (error) {
    console.error("Failed to export data:", error);
    throw error;
  }
}
