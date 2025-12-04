/**
 * Grid Filter Service
 * Handles filtering logic and filter UI management
 */

/**
 * Apply column filter from modal
 */
export function applyColumnFilter(
  column,
  selectedValues,
  columnFilters,
  tabId,
  tabDataStore,
  onFilterApplied
) {
  if (!column) return;

  if (selectedValues && selectedValues.size > 0) {
    columnFilters[column] = Array.from(selectedValues);
  } else {
    delete columnFilters[column];
  }

  columnFilters = { ...columnFilters };

  if (tabId) {
    tabDataStore.setFilters(tabId, columnFilters);
  }

  // Trigger reload
  if (onFilterApplied) {
    onFilterApplied();
  }

  return columnFilters;
}

/**
 * Clear specific column filter
 */
export function clearColumnFilter(
  column,
  columnFilters,
  selectedFilterValues,
  tabId,
  tabDataStore,
  onFilterCleared
) {
  if (!column) return;

  delete columnFilters[column];
  delete selectedFilterValues[column];
  columnFilters = { ...columnFilters };
  selectedFilterValues = { ...selectedFilterValues };

  if (tabId) {
    tabDataStore.setFilters(tabId, columnFilters);
  }

  if (onFilterCleared) {
    onFilterCleared();
  }

  return { columnFilters, selectedFilterValues };
}

/**
 * Clear all filters
 */
export function clearAllFilters(
  columnFilters,
  selectedFilterValues,
  tabId,
  tabDataStore,
  onAllFiltersCleared
) {
  columnFilters = {};
  selectedFilterValues = {};

  if (tabId) {
    tabDataStore.setFilters(tabId, columnFilters);
  }

  if (onAllFiltersCleared) {
    onAllFiltersCleared();
  }

  return { columnFilters, selectedFilterValues };
}

/**
 * Open filter modal
 */
export function openFilterModal(
  column,
  event,
  columnFilters,
  selectedFilterValues
) {
  if (!event || !event.target) {
    console.warn("⚠️ Invalid event for openFilterModal");
    return null;
  }

  const thElement = event.target.closest("th");
  if (!thElement) {
    console.warn("⚠️ Could not find parent TH element");
    return null;
  }

  const rect = thElement.getBoundingClientRect();
  const position = {
    top: rect.bottom + 5,
    left: rect.left,
  };

  // Initialize selected values from current filter
  if (columnFilters[column] && Array.isArray(columnFilters[column])) {
    selectedFilterValues[column] = new Set(columnFilters[column]);
  } else {
    selectedFilterValues[column] = new Set();
  }

  return {
    column,
    position,
    selectedValues: selectedFilterValues[column],
  };
}
