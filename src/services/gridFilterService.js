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

/**
 * Handle filter text input keydown
 */
export function handleFilterKeydown(
  column,
  event,
  columnFilters,
  tabId,
  tabDataStore,
  onFilterApplied
) {
  if (event.key === "Enter") {
    const value = event.target.value.trim();

    if (value) {
      columnFilters[column] = value;
    } else {
      delete columnFilters[column];
    }

    columnFilters = { ...columnFilters };

    if (tabId) {
      tabDataStore.setFilters(tabId, columnFilters);
    }

    if (onFilterApplied) {
      onFilterApplied();
    }

    return columnFilters;
  }

  return columnFilters;
}

/**
 * Get distinct values from data (client-side)
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

/**
 * Prepare filters for query
 */
export function prepareFiltersForQuery(columnFilters) {
  const filters = [];
  for (const [column, value] of Object.entries(columnFilters)) {
    if (Array.isArray(value) && value.length > 0) {
      filters.push({
        column,
        operator: "in",
        value: value,
      });
    } else if (typeof value === "string" && value.trim() !== "") {
      filters.push({
        column,
        operator: "like",
        value: `%${value}%`,
      });
    }
  }
  return filters;
}
