/**
 * Confirmation dialog utilities for destructive actions
 */

/**
 * Shows a confirmation dialog for delete operations
 * @param {string} itemType - Type of item being deleted (e.g., "connection", "database", "table")
 * @param {string} itemName - Name of the item being deleted
 * @param {Object} options - Additional options
 * @param {boolean} options.isDestructive - Whether to show a destructive warning (default: false)
 * @param {string} options.customMessage - Custom confirmation message
 * @returns {boolean} - Whether the user confirmed the deletion
 */
export function confirmDelete(itemType, itemName, options = {}) {
  const { isDestructive = false, customMessage } = options;

  let message;
  if (customMessage) {
    message = customMessage;
  } else if (isDestructive) {
    message = `Are you sure you want to delete ${itemType} "${itemName}"?\n\nWARNING: This will permanently delete all data in this ${itemType}!`;
  } else {
    message = `Are you sure you want to delete ${itemType} "${itemName}"?`;
  }

  return confirm(message);
}

/**
 * Shows a "not implemented" alert for features that are still TODO
 * @param {string} actionName - Name of the action (e.g., "Database deletion", "Schema rename")
 */
export function showNotImplemented(actionName) {
  alert(`${actionName} is not yet implemented.`);
}
