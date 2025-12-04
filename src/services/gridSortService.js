/**
 * Grid Sort Service
 * Handles sorting logic
 */

/**
 * Handle sort column click
 */
export function handleSort(
  column,
  sortColumn,
  sortDirection,
  tabId,
  tabDataStore,
  onSortChanged
) {
  let newSortColumn = sortColumn;
  let newSortDirection = sortDirection;

  if (sortColumn === column) {
    newSortDirection = sortDirection === "asc" ? "desc" : "asc";
  } else {
    newSortColumn = column;
    newSortDirection = "asc";
  }

  // Save to store
  if (tabId) {
    tabDataStore.setSortState(tabId, newSortColumn, newSortDirection);
  }

  // Trigger callback
  if (onSortChanged) {
    onSortChanged();
  }

  return {
    sortColumn: newSortColumn,
    sortDirection: newSortDirection,
  };
}
