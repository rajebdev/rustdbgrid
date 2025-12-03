/**
 * Grid State Manager Utility
 * Handles saving and restoring grid state
 */

/**
 * Save tab state to store
 */
export function saveTabState(
  tabId,
  tabDataStore,
  scrollPosition,
  filters,
  sortColumn,
  sortDirection,
  viewMode
) {
  if (!tabId || !tabDataStore) return;

  if (scrollPosition !== undefined) {
    tabDataStore.setScrollPosition(tabId, scrollPosition);
  }

  if (filters !== undefined) {
    tabDataStore.setFilters(tabId, filters);
  }

  if (sortColumn !== undefined && sortDirection !== undefined) {
    tabDataStore.setSortState(tabId, sortColumn, sortDirection);
  }

  if (viewMode !== undefined) {
    tabDataStore.setViewMode(tabId, viewMode);
  }
}

/**
 * Save scroll position
 */
export function saveScrollPosition(tabId, tabDataStore, scrollTop) {
  if (!tabId || !tabDataStore || scrollTop === undefined) return;
  tabDataStore.setScrollPosition(tabId, scrollTop);
}

/**
 * Restore scroll position
 */
export function restoreScrollPosition(
  tableWrapper,
  rowNumbersWrapper,
  savedScrollPosition,
  isRestoringScroll,
  setIsRestoringScroll,
  delay = 100
) {
  if (!savedScrollPosition || isRestoringScroll) return;

  if (!tableWrapper) {
    console.warn("⚠️ tableWrapper is not available for scroll restore");
    return;
  }

  setIsRestoringScroll(true);

  setTimeout(() => {
    if (tableWrapper) {
      tableWrapper.scrollTop = savedScrollPosition || 0;
      if (rowNumbersWrapper) {
        rowNumbersWrapper.scrollTop = savedScrollPosition || 0;
      }
    }
    setIsRestoringScroll(false);
  }, delay);
}

/**
 * Clear all saved state
 */
export function clearTabState(tabId, tabDataStore) {
  if (!tabId || !tabDataStore) return;

  tabDataStore.clearTab(tabId);
}

/**
 * Get initial state from store
 */
export function getInitialTabState(tabId, tabDataStore) {
  if (!tabId || !tabDataStore) return null;

  const store = tabDataStore[tabId];
  if (!store) return null;

  return {
    filters: store.filters || {},
    sortColumn: store.sortColumn || null,
    sortDirection: store.sortDirection || "asc",
    viewMode: store.viewMode || "grid",
    scrollPosition: store.scrollPosition || 0,
  };
}
