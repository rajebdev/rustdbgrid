<script>
  import { onMount, onDestroy } from "svelte";
  import { tabDataStore } from "../../stores/tabData";
  import { activeConnection } from "../../stores/connections";
  import { buildPaginatedQuery } from "../../utils/defaultQueries";
  import {
    getFilterValues,
    executeQueryWithFilters,
    executeQuery,
    getTableData,
  } from "../../utils/tauri";
  import ArrayCell from "./ArrayCell.svelte";

  export let data = null;
  export let tabId = null;
  export let executedQuery = "";
  export let connection = null;
  export let tableName = ""; // Table/cache name for Ignite/NoSQL
  export let databaseName = ""; // Database/cache name for Ignite/NoSQL

  let displayData = null; // Internal state for display (can be filtered or original)
  let isFiltered = false; // Track if current data is filtered
  let totalRows = 0; // Total rows in database (from COUNT query)
  let currentOffset = 0; // Track current offset for pagination
  let isLoadingMore = false; // Track if loading more data
  let hasMoreData = true; // Track if there's more data to load
  let lastLoadedQueryId = null; // Track which query the data belongs to
  let columnFilters = {};
  let sortColumn = null;
  let sortDirection = "asc";
  let tableWrapper;
  let showFilterModal = false;
  let filterModalColumn = null;
  let filterModalPosition = { top: 0, left: 0 };
  let filterSearchQuery = "";
  let selectedFilterValues = {};
  let filterValuesCache = {}; // Cache for server-side filter values
  let loadingFilterValues = false;
  let isLoadingData = false;
  let finalQuery = ""; // Store the final executed query with filters
  let currentTabId = null; // Track current tab to prevent unnecessary restores
  let isRestoringScroll = false; // Flag to prevent restore during user scroll
  let viewMode = "grid"; // View mode: "grid" or "json"

  // Inline editing state
  let editingCell = null; // { rowIndex, column }
  let editingValue = "";
  let originalValue = "";
  let editedRows = new Map(); // Map<rowIndex, Map<column, newValue>>
  let originalRowData = new Map(); // Map<rowIndex, originalRowObject> - backup for cancel
  let showSqlPreview = false;
  let previewSql = "";
  let pendingUpdates = [];
  let showPopupEditor = false;
  let popupEditorValue = "";
  let popupEditingCell = null;

  // Reactive variable to track if there are edits
  $: hasUnsavedEdits = editedRows.size > 0;

  // Set default view mode based on database type
  $: if (connection && !$tabDataStore[tabId]?.viewMode) {
    viewMode = connection.db_type === "MongoDB" ? "json" : "grid";
  }

  // Load saved state when tab changes (only when tabId actually changes)
  $: if (tabId && tabId !== currentTabId) {
    currentTabId = tabId;

    if ($tabDataStore[tabId]) {
      const savedState = $tabDataStore[tabId];
      columnFilters = savedState.filters || {};
      sortColumn = savedState.sortColumn || null;
      sortDirection = savedState.sortDirection || "asc";
      viewMode =
        savedState.viewMode ||
        (connection?.db_type === "MongoDB" ? "json" : "grid");

      // Restore scroll position after a short delay, only once per tab change
      if (tableWrapper && savedState.scrollPosition) {
        isRestoringScroll = true;
        setTimeout(() => {
          if (tableWrapper) {
            tableWrapper.scrollTop = savedState.scrollPosition || 0;
            lastScrollTop = savedState.scrollPosition || 0;
          }
          isRestoringScroll = false;
        }, 100);
      }
    }
  }

  // Update displayData when original data changes (only if not filtered and not loading more)
  $: if (data && !isFiltered && !isLoadingMore) {
    const queryId =
      executedQuery +
      JSON.stringify(columnFilters) +
      sortColumn +
      sortDirection;
    // Only reset if this is a new query (not a load more operation)
    if (queryId !== lastLoadedQueryId) {
      displayData = data;
      totalRows = data?.total_count || data?.rows?.length || 0;
      currentOffset = data?.rows?.length || 0; // Set offset to current row count
      hasMoreData = data?.rows?.length === 200; // Check if we got full batch
      lastLoadedQueryId = queryId;
    }
  }

  // Clear cache when executedQuery changes
  $: if (executedQuery) {
    filterValuesCache = {};
    columnFilters = {};
    sortColumn = null;
    sortDirection = "asc";
    currentOffset = 0;
    hasMoreData = true;
    lastLoadedQueryId = null; // Reset query ID tracking
  }

  async function reloadDataWithFilters() {
    // Validate executedQuery is not empty
    if (!executedQuery || executedQuery.trim() === "") {
      console.log("⚠️ Cannot reload - executedQuery is empty");
      return;
    }

    if (!connection) {
      console.log("⚠️ Cannot reload - no active connection");
      return;
    }

    // Check if we have filters or sorting applied
    const hasFilters = Object.keys(columnFilters).length > 0;
    const hasSorting = sortColumn !== null;

    // Clear current data and show loading state
    const previousData = displayData;
    displayData = null; // Clear data immediately
    isLoadingData = true; // Set loading state
    isFiltered = hasFilters || hasSorting; // Mark as filtered only if we have filters or sorting
    currentOffset = 0; // Reset offset
    hasMoreData = true; // Reset hasMoreData

    // Force UI update with a small delay
    await new Promise((resolve) => setTimeout(resolve, 0));

    try {
      console.log("🔄 Reloading with filters:", {
        executedQuery: executedQuery.substring(0, 100),
        columnFilters,
        sortColumn,
        sortDirection,
      });

      // Convert columnFilters to the format expected by backend
      const filters = {};
      for (const [col, value] of Object.entries(columnFilters)) {
        // Support both array (from modal) and string (from text input)
        if (Array.isArray(value) && value.length > 0) {
          filters[col] = value;
        } else if (typeof value === "string" && value.trim() !== "") {
          filters[col] = value;
        }
      }

      console.log("📦 Filters to send:", filters);

      const dbType = connection?.db_type || "MySQL";
      const isIgnite = dbType === "Ignite";

      let result;

      // For Apache Ignite, ALWAYS use SCAN - never use SQL query
      if (isIgnite) {
        // Priority: databaseName prop > tableName prop > parse from executedQuery
        let cacheName = databaseName || tableName || "";

        // Fallback: try to parse from SCAN query if props are empty
        if (!cacheName) {
          const scanMatch = executedQuery.match(/^SCAN\s+(\S+)/i);
          cacheName = scanMatch ? scanMatch[1] : "";
        }

        console.log("🔥 Ignite reload with SCAN:", {
          cacheName,
          databaseName,
          tableName,
          executedQuery: executedQuery.substring(0, 50),
        });

        if (cacheName) {
          // Use getTableData which uses SCAN internally
          result = await getTableData(connection, cacheName, "", 200, 0);

          // Apply client-side filtering for Ignite if filters are set
          if (Object.keys(filters).length > 0 && result?.rows) {
            const filteredRows = result.rows.filter((row) => {
              return Object.entries(filters).every(([column, filterValue]) => {
                const cellValue = row[column];
                const cellStr =
                  cellValue === null || cellValue === undefined
                    ? "NULL"
                    : String(cellValue);

                if (Array.isArray(filterValue)) {
                  // Array filter - check if value is in the array
                  return filterValue.some((fv) => {
                    if (fv === "NULL")
                      return cellValue === null || cellValue === undefined;
                    return cellStr === fv;
                  });
                } else if (
                  typeof filterValue === "string" &&
                  filterValue.trim() !== ""
                ) {
                  // Text filter - case-insensitive contains
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
          if (sortColumn && result?.rows) {
            const direction = sortDirection.toUpperCase() === "DESC" ? -1 : 1;
            result.rows.sort((a, b) => {
              const aVal = a[sortColumn];
              const bVal = b[sortColumn];
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
        result = await executeQueryWithFilters(
          connection,
          executedQuery,
          Object.keys(filters).length > 0 ? filters : null,
          sortColumn,
          sortColumn ? sortDirection.toUpperCase() : null,
          200, // Default limit 200 rows
          0 // Start from offset 0
        );
      }

      // Update displayData with filtered results
      displayData = result;
      totalRows = result?.total_count || result?.rows?.length || 0;
      currentOffset = result?.rows?.length || 0;
      hasMoreData = result?.rows?.length === 200;
      lastLoadedQueryId =
        executedQuery + JSON.stringify(filters) + sortColumn + sortDirection;

      // Store the final query from backend
      if (result.final_query) {
        finalQuery = result.final_query;
      }
    } catch (error) {
      console.error("❌ Failed to reload data with filters:", error);
      // Restore previous data on error
      displayData = previousData;
      isFiltered = false;
    } finally {
      isLoadingData = false;
    }
  }

  async function loadMoreData() {
    if (isLoadingMore || !hasMoreData || !executedQuery || !connection) {
      return;
    }

    isLoadingMore = true;

    try {
      console.log("📥 Loading more data, offset:", currentOffset);

      // Convert columnFilters to the format expected by backend
      const filters = {};
      for (const [col, value] of Object.entries(columnFilters)) {
        if (Array.isArray(value) && value.length > 0) {
          filters[col] = value;
        } else if (typeof value === "string" && value.trim() !== "") {
          filters[col] = value;
        }
      }

      // Build query with LIMIT and OFFSET based on database type
      const dbType = connection?.db_type || "MySQL";

      // For Apache Ignite, ALWAYS use SCAN - never use SQL query
      const isIgnite = dbType === "Ignite";

      let result;
      let loadMoreQuery = executedQuery;

      if (isIgnite) {
        // For Ignite, ALWAYS use SCAN with getTableData
        // Priority: databaseName prop > tableName prop > parse from executedQuery
        let cacheName = databaseName || tableName || "";

        // Fallback: try to parse from SCAN query if props are empty
        if (!cacheName) {
          const scanMatch = executedQuery.match(/^SCAN\s+(\S+)/i);
          cacheName = scanMatch ? scanMatch[1] : "";
        }

        console.log("📥 Ignite SCAN load more:", {
          cacheName,
          databaseName,
          tableName,
          offset: currentOffset,
          executedQuery: executedQuery.substring(0, 50),
        });

        if (!cacheName) {
          console.error("❌ Cannot determine cache name for Ignite load more");
          hasMoreData = false;
          isLoadingMore = false;
          return;
        }

        result = await getTableData(
          connection,
          cacheName, // database = cache name
          "", // table = empty for cache scan
          200, // limit
          currentOffset // offset
        );

        loadMoreQuery = `SCAN ${cacheName} LIMIT 200 OFFSET ${currentOffset}`;
      } else {
        // Standard SQL-based pagination for other databases
        // Check if we have filters - if so, let backend handle pagination with offset
        const hasFilters = Object.keys(filters).length > 0;

        if (hasFilters) {
          // With filters: send original query + offset, backend handles pagination
          result = await executeQueryWithFilters(
            connection,
            executedQuery, // Original query without pagination
            filters,
            sortColumn,
            sortColumn ? sortDirection.toUpperCase() : null,
            200,
            currentOffset // Send offset to backend
          );
          loadMoreQuery = result.final_query || executedQuery;
        } else {
          // Without filters: build paginated query in frontend
          loadMoreQuery = buildPaginatedQuery(
            dbType,
            executedQuery,
            200,
            currentOffset
          );

          result = await executeQueryWithFilters(
            connection,
            loadMoreQuery,
            null,
            sortColumn,
            sortColumn ? sortDirection.toUpperCase() : null,
            null, // No limit needed, already in query
            null // No offset needed, already in query
          );
        }
      }

      if (result && result.rows && result.rows.length > 0) {
        // Append new rows to existing data
        const newRows = [...(displayData?.rows || []), ...result.rows];
        displayData = {
          ...displayData,
          rows: newRows,
          execution_time: result.execution_time,
        };

        currentOffset = newRows.length; // Set offset to total row count
        hasMoreData = result.rows.length === 200; // If less than 200, no more data

        // Update final query - prefer backend's final_query to preserve original format for NoSQL
        if (result.final_query) {
          finalQuery = result.final_query;
        } else {
          finalQuery = loadMoreQuery;
        }

        console.log(
          "✅ Loaded",
          result.rows.length,
          "more rows. Total:",
          newRows.length,
          "| Next offset:",
          currentOffset
        );
      } else {
        hasMoreData = false;
        console.log("✅ No more data to load");
      }
    } catch (error) {
      console.error("❌ Failed to load more data:", error);
      hasMoreData = false;
    } finally {
      isLoadingMore = false;
    }
  }

  // Save scroll position when scrolling with throttle
  let scrollTimeout;
  let lastScrollTop = 0;
  let lastLoadTriggeredAt = 0; // Track last time load was triggered

  function handleScroll() {
    if (!tableWrapper || isRestoringScroll) return;

    const currentScrollTop = tableWrapper.scrollTop;

    // Check if scrolled near bottom (within 200px)
    const scrollHeight = tableWrapper.scrollHeight;
    const clientHeight = tableWrapper.clientHeight;
    const distanceFromBottom = scrollHeight - (currentScrollTop + clientHeight);
    const scrolledToBottom = distanceFromBottom < 200;

    // Only trigger load if:
    // 1. Scrolled to bottom
    // 2. Has more data
    // 3. Not currently loading
    // 4. Scrolling down (not up)
    // 5. Haven't triggered load recently (within 1 second)
    const now = Date.now();
    const isScrollingDown = currentScrollTop > lastScrollTop;
    const canTriggerLoad = now - lastLoadTriggeredAt > 1000;

    if (
      scrolledToBottom &&
      hasMoreData &&
      !isLoadingMore &&
      isScrollingDown &&
      canTriggerLoad
    ) {
      lastLoadTriggeredAt = now;
      loadMoreData();
    }

    // Only update if scroll changed significantly (more than 5px)
    if (Math.abs(currentScrollTop - lastScrollTop) < 5) {
      return;
    }

    lastScrollTop = currentScrollTop;

    // Clear previous timeout
    if (scrollTimeout) {
      clearTimeout(scrollTimeout);
    }

    // Throttle scroll save to prevent excessive updates
    scrollTimeout = setTimeout(() => {
      if (tabId && tableWrapper) {
        tabDataStore.setScrollPosition(tabId, tableWrapper.scrollTop);
      }
    }, 150);
  }
  function formatValue(value) {
    if (value === null || value === undefined) {
      return "NULL";
    }
    if (Array.isArray(value)) {
      // Format array nicely: [item1, item2, ...]
      if (value.length === 0) {
        return "[]";
      }
      // For arrays of primitives, show comma-separated
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

  function isNumericColumn(column) {
    if (!data?.rows || data.rows.length === 0) return false;

    // Sample first 10 rows to determine if column is numeric
    const sampleSize = Math.min(10, data.rows.length);
    let numericCount = 0;

    for (let i = 0; i < sampleSize; i++) {
      const value = data.rows[i][column];
      if (value === null || value === undefined) continue;
      // Only consider actual numbers, not strings that look like numbers
      if (typeof value === "number") {
        numericCount++;
      }
    }

    // If more than 70% are numeric, consider it a numeric column
    return numericCount / sampleSize > 0.7;
  }

  function handleSort(column) {
    if (sortColumn === column) {
      sortDirection = sortDirection === "asc" ? "desc" : "asc";
    } else {
      sortColumn = column;
      sortDirection = "asc";
    }

    // Save to store
    if (tabId) {
      tabDataStore.setSortState(tabId, sortColumn, sortDirection);
    }

    // Trigger server-side reload
    reloadDataWithFilters();
  }

  function handleFilterKeydown(column, event) {
    if (event.key === "Enter") {
      const value = event.target.value.trim();

      if (value) {
        // Set as string filter (for LIKE query)
        columnFilters[column] = value;
      } else {
        // Clear filter if empty
        delete columnFilters[column];
      }

      columnFilters = { ...columnFilters };

      if (tabId) {
        tabDataStore.setFilters(tabId, columnFilters);
      }

      // Trigger server-side reload
      reloadDataWithFilters();
    }
  }

  function updateFilter(column, value) {
    // This function is kept for compatibility but filters are applied via modal
    // Text input is now read-only when array filter is active
  }

  function applyColumnFilter() {
    if (!filterModalColumn) return;

    const selected = selectedFilterValues[filterModalColumn];
    if (selected && selected.size > 0) {
      columnFilters[filterModalColumn] = Array.from(selected);
    } else {
      delete columnFilters[filterModalColumn];
    }

    columnFilters = { ...columnFilters };

    if (tabId) {
      tabDataStore.setFilters(tabId, columnFilters);
    }

    closeFilterModal();

    // Trigger server-side reload
    reloadDataWithFilters();
  }

  function clearColumnFilter() {
    if (!filterModalColumn) return;

    delete columnFilters[filterModalColumn];
    delete selectedFilterValues[filterModalColumn];
    columnFilters = { ...columnFilters };
    selectedFilterValues = { ...selectedFilterValues };

    if (tabId) {
      tabDataStore.setFilters(tabId, columnFilters);
    }

    // Reset final query
    finalQuery = "";

    closeFilterModal();

    // Trigger server-side reload
    reloadDataWithFilters();
  }

  function openFilterModal(column, event) {
    const rect = event.target.closest("th").getBoundingClientRect();
    filterModalPosition = {
      top: rect.bottom + 5,
      left: rect.left,
    };
    filterModalColumn = column;
    filterSearchQuery = "";

    // Initialize selected values from current filter
    if (columnFilters[column] && Array.isArray(columnFilters[column])) {
      selectedFilterValues[column] = new Set(columnFilters[column]);
    } else {
      selectedFilterValues[column] = new Set();
    }

    console.log("🔍 Opening filter modal for column:", column);
    console.log("📝 executedQuery:", executedQuery);
    console.log("🔌 connection:", connection);

    showFilterModal = true;

    // Load filter values from server
    loadFilterValuesFromServer(column);
  }

  async function loadFilterValuesFromServer(column, search = null) {
    const cacheKey = `${column}_${search || ""}`;

    // Check cache first
    if (filterValuesCache[cacheKey]) {
      console.log("✅ Using cached filter values for", cacheKey);
      return;
    }

    // Check if we have query and connection
    if (!executedQuery || executedQuery.trim() === "" || !connection) {
      console.log("⚠️ Skipping server-side filter:");
      console.log(
        "  - executedQuery:",
        executedQuery ? `"${executedQuery.substring(0, 50)}..."` : "EMPTY"
      );
      console.log("  - connection:", connection ? "EXISTS" : "NULL");
      // Fallback to client-side filtering
      filterValuesCache[cacheKey] = getDistinctValues(column);
      filterValuesCache = { ...filterValuesCache };
      return;
    }

    loadingFilterValues = true;
    console.log(
      "🔍 Loading filter values from SERVER for column:",
      column,
      "search:",
      search,
      "\nQuery:",
      executedQuery.substring(0, 100)
    );

    try {
      const result = await getFilterValues(
        connection,
        executedQuery,
        column,
        search,
        1000
      );

      console.log("✅ Server-side filter result:", result);
      filterValuesCache[cacheKey] = result.values;
      // Force reactivity update
      filterValuesCache = { ...filterValuesCache };
    } catch (error) {
      console.error("❌ Failed to load filter values:", error);
      // Fallback to client-side filtering
      filterValuesCache[cacheKey] = getDistinctValues(column);
      filterValuesCache = { ...filterValuesCache };
    } finally {
      loadingFilterValues = false;
    }
  }
  function closeFilterModal() {
    showFilterModal = false;
    filterModalColumn = null;
    filterSearchQuery = "";
  }

  function getDistinctValues(column) {
    if (!data?.rows) return [];
    const values = new Set();
    data.rows.forEach((row) => {
      values.add(formatValue(row[column]));
    });
    return Array.from(values).sort();
  }

  function toggleFilterValue(column, value) {
    if (!selectedFilterValues[column]) {
      selectedFilterValues[column] = new Set();
    }

    if (selectedFilterValues[column].has(value)) {
      selectedFilterValues[column].delete(value);
    } else {
      selectedFilterValues[column].add(value);
    }
    selectedFilterValues = { ...selectedFilterValues };
  }

  function selectAllFilterValues() {
    if (!filterModalColumn) return;
    const cacheKey = `${filterModalColumn}_${filterSearchQuery || ""}`;
    const values =
      filterValuesCache[cacheKey] || getDistinctValues(filterModalColumn);
    const filtered = filterSearchQuery
      ? values.filter((v) =>
          v.toLowerCase().includes(filterSearchQuery.toLowerCase())
        )
      : values;

    selectedFilterValues[filterModalColumn] = new Set(filtered);
    selectedFilterValues = { ...selectedFilterValues };
  }

  function deselectAllFilterValues() {
    if (!filterModalColumn) return;
    selectedFilterValues[filterModalColumn] = new Set();
    selectedFilterValues = { ...selectedFilterValues };
  }

  function clearAllFilters() {
    columnFilters = {};
    selectedFilterValues = {};
    sortColumn = null;
    sortDirection = "asc";
    isFiltered = false; // Clear filtered flag
    currentOffset = 0; // Reset offset
    hasMoreData = true;

    // Reset final query
    finalQuery = "";

    if (tabId) {
      tabDataStore.setFilters(tabId, {});
      tabDataStore.setSortState(tabId, null, "asc");
    }

    // Restore original data by resetting to the base data
    displayData = data;
    totalRows = data?.total_count || data?.rows?.length || 0;
    currentOffset = data?.rows?.length || 0;
    hasMoreData = data?.rows?.length === 200;

    // Update the query ID to match the clean state
    lastLoadedQueryId = executedQuery + JSON.stringify({}) + null + "asc";
  }

  // Watch for search query changes and load from server
  let searchDebounceTimeout;
  $: if (showFilterModal && filterModalColumn) {
    clearTimeout(searchDebounceTimeout);
    searchDebounceTimeout = setTimeout(() => {
      if (executedQuery && connection) {
        loadFilterValuesFromServer(
          filterModalColumn,
          filterSearchQuery || null
        );
      }
    }, 300);
  }

  // Display rows directly - filtering and sorting should be done on server-side
  $: displayRows = displayData?.rows || [];

  // Auto-focus action for inline editing
  function focusInput(node) {
    node.focus();
    node.select();
  }

  function toggleViewMode() {
    viewMode = viewMode === "grid" ? "json" : "grid";
    if (tabId) {
      tabDataStore.setViewMode(tabId, viewMode);
    }
  }

  // Inline editing functions
  function startEdit(rowIndex, column, currentValue) {
    let valueStr;
    if (currentValue === null || currentValue === undefined) {
      valueStr = "";
    } else if (Array.isArray(currentValue)) {
      // Format array as JSON for editing
      valueStr = JSON.stringify(currentValue);
    } else if (typeof currentValue === "object") {
      valueStr = JSON.stringify(currentValue);
    } else {
      valueStr = String(currentValue);
    }

    // Check if the value is longer than 500px (estimate ~70 characters for typical width)
    // or if it contains newlines (multi-line text)
    const estimatedWidth = valueStr.length * 7; // Rough estimate: 7px per character
    const hasNewlines = valueStr.includes("\n") || valueStr.includes("\r");

    if (estimatedWidth > 500 || hasNewlines || valueStr.length > 70) {
      // Show popup editor for long content
      showPopupEditor = true;
      popupEditorValue = valueStr;
      popupEditingCell = { rowIndex, column };
      originalValue = currentValue;
    } else {
      // Use inline editing for short content
      editingCell = { rowIndex, column };
      editingValue = valueStr;
      originalValue = currentValue;
    }
  }

  function cancelEdit() {
    editingCell = null;
    editingValue = "";
    originalValue = "";
  }

  function closePopupEditor() {
    showPopupEditor = false;
    popupEditorValue = "";
    popupEditingCell = null;
    originalValue = "";
  }

  function savePopupEdit() {
    if (!popupEditingCell) return;

    const { rowIndex, column } = popupEditingCell;

    // Check if value actually changed
    if (
      popupEditorValue === originalValue ||
      (popupEditorValue === "" &&
        (originalValue === null || originalValue === undefined))
    ) {
      closePopupEditor();
      return;
    }

    // Backup original row data if this is the first edit on this row
    if (!originalRowData.has(rowIndex)) {
      originalRowData.set(rowIndex, { ...displayData.rows[rowIndex] });
    }

    // Store the edit in editedRows Map
    if (!editedRows.has(rowIndex)) {
      editedRows.set(rowIndex, new Map());
    }
    editedRows.get(rowIndex).set(column, popupEditorValue);

    // Update display
    displayData.rows[rowIndex][column] = popupEditorValue;
    displayData = { ...displayData };

    // Trigger reactivity by creating new Map
    editedRows = new Map(editedRows);

    closePopupEditor();
  }

  function finishEdit() {
    console.log("🔧 finishEdit() called", {
      editingCell,
      editingValue,
      originalValue,
    });

    if (!editingCell) {
      console.log("⚠️ No editing cell, returning");
      return;
    }

    const { rowIndex, column } = editingCell;

    // Check if value actually changed
    if (
      editingValue === originalValue ||
      (editingValue === "" &&
        (originalValue === null || originalValue === undefined))
    ) {
      console.log("⚠️ Value not changed, canceling edit");
      cancelEdit();
      return;
    }

    console.log("✅ Value changed, storing edit", {
      rowIndex,
      column,
      editingValue,
    });

    // Backup original row data if this is the first edit on this row
    if (!originalRowData.has(rowIndex)) {
      originalRowData.set(rowIndex, { ...displayData.rows[rowIndex] });
    }

    // Store the edit in editedRows Map
    if (!editedRows.has(rowIndex)) {
      editedRows.set(rowIndex, new Map());
    }
    editedRows.get(rowIndex).set(column, editingValue);

    console.log(
      "📝 editedRows before reactivity:",
      editedRows.size,
      Array.from(editedRows.entries())
    );

    // Update display
    displayData.rows[rowIndex][column] = editingValue;
    displayData = { ...displayData };

    // Trigger reactivity by creating new Map
    editedRows = new Map(editedRows);

    console.log(
      "✨ editedRows after reactivity:",
      editedRows.size,
      Array.from(editedRows.entries())
    );
    console.log("🔍 hasEdits():", hasEdits());

    cancelEdit();
  }

  function hasEdits() {
    return editedRows.size > 0;
  }

  function cancelAllEdits() {
    console.log("🔄 Canceling all edits, restoring original data...");

    // Restore original values from backup
    originalRowData.forEach((originalRow, rowIndex) => {
      console.log(`  Restoring row ${rowIndex}:`, originalRow);
      displayData.rows[rowIndex] = { ...originalRow };
    });

    // Force reactivity
    displayData = { ...displayData };

    // Clear all edits and backups
    editedRows.clear();
    editedRows = new Map();
    originalRowData.clear();
    originalRowData = new Map();

    console.log("✅ All edits canceled, data restored");

    cancelEdit();
  }

  function generateUpdateSql() {
    if (!displayData || !displayData.rows || !executedQuery) {
      return [];
    }

    const updates = [];

    // Extract table name from query - support table.schema notation
    // Match: FROM `schema`.`table`, FROM schema.table, FROM `table`, FROM table
    let tableMatch = executedQuery.match(/FROM\s+`?(\w+)`?\.`?(\w+)`?/i);
    let tableName = "";

    if (tableMatch) {
      // Has schema.table format
      tableName = `\`${tableMatch[1]}\`.\`${tableMatch[2]}\``;
      console.log("📋 Extracted table name (with schema):", tableName);
    } else {
      // Try simple table name
      tableMatch = executedQuery.match(/FROM\s+`?(\w+)`?/i);
      tableName = tableMatch ? `\`${tableMatch[1]}\`` : "`table`";
      console.log("📋 Extracted table name (simple):", tableName);
    }

    console.log("📊 Available column_types:", displayData.column_types);

    editedRows.forEach((changes, rowIndex) => {
      const row = displayData.rows[rowIndex];
      if (!row) return;

      const setClauses = [];
      const whereClauses = [];

      // Build SET clause
      changes.forEach((newValue, column) => {
        const sqlValue =
          newValue === "" ? "NULL" : `'${newValue.replace(/'/g, "''")}'`;
        setClauses.push(`\`${column}\` = ${sqlValue}`);
      });

      // Build WHERE clause - use all columns that weren't edited as identifiers
      displayData.columns.forEach((column) => {
        if (
          !changes.has(column) &&
          row[column] !== null &&
          row[column] !== undefined
        ) {
          const value = row[column];
          const columnType =
            displayData.column_types?.[column]?.toUpperCase() || "";
          let sqlValue;

          // Check column type from metadata
          if (
            columnType.includes("INT") ||
            columnType.includes("DECIMAL") ||
            columnType.includes("NUMERIC") ||
            columnType.includes("FLOAT") ||
            columnType.includes("DOUBLE") ||
            columnType.includes("REAL")
          ) {
            // Numeric types - no quotes
            sqlValue = value;
            console.log(`  ${column}: ${value} (${columnType}, no quotes)`);
          } else if (
            columnType.includes("BOOL") ||
            columnType.includes("BIT")
          ) {
            // Boolean - convert to 1/0
            sqlValue = value ? 1 : 0;
            console.log(`  ${column}: ${value} (${columnType} -> ${sqlValue})`);
          } else if (
            columnType.includes("DATE") ||
            columnType.includes("TIME") ||
            columnType.includes("TIMESTAMP")
          ) {
            // Date/Time types - quote them
            sqlValue = `'${String(value).replace(/'/g, "''")}'`;
            console.log(`  ${column}: "${value}" (${columnType}, quoted)`);
          } else {
            // String types (VARCHAR, TEXT, CHAR, etc.) - quote them
            sqlValue = `'${String(value).replace(/'/g, "''")}'`;
            console.log(`  ${column}: "${value}" (${columnType}, quoted)`);
          }

          whereClauses.push(`\`${column}\` = ${sqlValue}`);
        }
      });

      if (setClauses.length > 0 && whereClauses.length > 0) {
        const sql = `UPDATE ${tableName} SET ${setClauses.join(", ")} WHERE ${whereClauses.join(" AND ")};`;
        console.log("✅ Generated SQL:", sql);
        updates.push({
          sql,
          rowIndex,
        });
      }
    });

    return updates;
  }

  function showPreview() {
    pendingUpdates = generateUpdateSql();
    if (pendingUpdates.length > 0) {
      previewSql = pendingUpdates.map((u) => u.sql).join("\n");
      showSqlPreview = true;
    }
  }

  function closeSqlPreview() {
    showSqlPreview = false;
    previewSql = "";
    pendingUpdates = [];
  }

  async function executeUpdates() {
    if (pendingUpdates.length === 0 || !connection) return;

    try {
      // Execute each update
      for (const update of pendingUpdates) {
        await executeQuery(connection, update.sql);
      }

      // Clear edited rows after successful update
      editedRows.clear();
      editedRows = new Map();

      // Reload data
      await reloadDataWithFilters();

      closeSqlPreview();

      // Show success message (you can add a toast/notification here)
      alert("Updates executed successfully!");
    } catch (error) {
      console.error("Failed to execute updates:", error);
      alert(`Failed to execute updates: ${error}`);
    }
  }

  // Function to stringify JSON with consistent key order based on column order
  function stringifyRowWithOrder(row, columns) {
    const orderedRow = {};
    // Add keys in the order they appear in columns array
    columns.forEach((column) => {
      orderedRow[column] = row[column];
    });
    return JSON.stringify(orderedRow, null, 2);
  }
</script>

<div class="data-grid-container h-100 d-flex flex-column">
  {#if isLoadingData}
    <div
      class="d-flex flex-column align-items-center justify-content-center h-100 text-primary"
    >
      <i class="fas fa-spinner fa-spin fa-3x mb-3"></i>
      <p class="fs-5">Loading filtered data...</p>
    </div>
  {:else if displayData && displayData.rows.length > 0}
    <div class="d-flex align-items-center gap-2 p-2 bg-light border-bottom">
      <span class="badge bg-primary">
        <i class="fas fa-table"></i>
        {#if totalRows > displayRows.length}
          Rows: {displayRows.length.toLocaleString()} of {totalRows.toLocaleString()}
        {:else}
          Rows: {displayRows.length.toLocaleString()}
        {/if}
      </span>
      <span class="badge bg-info">
        <i class="fas fa-columns"></i> Columns: {displayData.columns.length}
      </span>
      <span class="badge bg-secondary">
        <i class="fas fa-clock"></i>
        {displayData.execution_time}ms
      </span>

      <!-- View Mode Toggle -->
      <div class="btn-group ms-auto" role="group">
        <button
          type="button"
          class="btn btn-sm {viewMode === 'grid'
            ? 'btn-primary'
            : 'btn-outline-primary'}"
          on:click={() => viewMode === "json" && toggleViewMode()}
          title="Grid View"
        >
          <i class="fas fa-table"></i> Grid
        </button>
        <button
          type="button"
          class="btn btn-sm {viewMode === 'json'
            ? 'btn-primary'
            : 'btn-outline-primary'}"
          on:click={() => viewMode === "grid" && toggleViewMode()}
          title="JSON View"
        >
          <i class="fas fa-code"></i> JSON
        </button>
      </div>

      {#if Object.keys(columnFilters).length > 0}
        <button class="btn btn-sm btn-danger" on:click={clearAllFilters}>
          <i class="fas fa-times"></i> Clear filters
        </button>
      {/if}
    </div>

    {#if viewMode === "grid"}
      <div
        class="table-container flex-grow-1"
        bind:this={tableWrapper}
        on:scroll={handleScroll}
      >
        <table
          class="table table-sm table-bordered data-table mb-0"
          style="table-layout: auto;"
        >
          <thead
            style="background-color: #e7f1ff; position: sticky; top: 0; z-index: 10; box-shadow: 0 2px 2px -1px rgba(0,0,0,0.1);"
          >
            <tr>
              {#each displayData.columns as column}
                {@const isNumeric = isNumericColumn(column)}
                <th>
                  <div class="column-header">
                    <button
                      class="sort-button"
                      on:click={() => handleSort(column)}
                    >
                      <span class="column-name">{column}</span>
                      {#if sortColumn === column}
                        <i
                          class="fas fa-sort-{sortDirection === 'asc'
                            ? 'up'
                            : 'down'} sort-icon"
                        ></i>
                      {:else}
                        <i class="fas fa-sort sort-icon inactive"></i>
                      {/if}
                    </button>
                    <button
                      class="filter-icon-button"
                      class:active={columnFilters[column]}
                      on:click={(e) => openFilterModal(column, e)}
                      title="Filter column"
                    >
                      <i class="fas fa-filter"></i>
                    </button>
                  </div>
                  <div class="p-1">
                    <input
                      type="text"
                      class="form-control form-control-sm"
                      placeholder="Type and press Enter..."
                      on:keydown={(e) => handleFilterKeydown(column, e)}
                      value={Array.isArray(columnFilters[column])
                        ? `${columnFilters[column].length} selected`
                        : columnFilters[column] || ""}
                      readonly={Array.isArray(columnFilters[column])}
                      title="Type text and press Enter to filter"
                    />
                  </div>
                </th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each displayRows as row, index}
              <tr class={editedRows.has(index) ? "edited-row" : ""}>
                {#each displayData.columns as column}
                  {@const isEdited =
                    editedRows.has(index) && editedRows.get(index).has(column)}
                  {@const cellValue = row[column]}
                  {@const isArrayValue = Array.isArray(cellValue)}
                  <td
                    class="{cellValue === null || cellValue === undefined
                      ? 'null-value fst-italic'
                      : ''} {isNumericColumn(column) && !isArrayValue
                      ? 'text-end font-monospace'
                      : ''} {editingCell?.rowIndex === index &&
                    editingCell?.column === column
                      ? 'editing'
                      : ''} {isEdited ? 'edited-cell' : ''} {isArrayValue
                      ? 'array-cell-td'
                      : ''}"
                    title={isArrayValue
                      ? `Array with ${cellValue.length} items`
                      : formatValue(cellValue)}
                    on:dblclick={() => startEdit(index, column, cellValue)}
                  >
                    {#if editingCell?.rowIndex === index && editingCell?.column === column}
                      <input
                        type="text"
                        class="form-control form-control-sm"
                        bind:value={editingValue}
                        on:keydown={(e) => {
                          if (e.key === "Enter") {
                            finishEdit();
                          } else if (e.key === "Escape") {
                            cancelEdit();
                          }
                        }}
                        on:blur={finishEdit}
                        use:focusInput
                      />
                    {:else if isArrayValue}
                      <ArrayCell value={cellValue} />
                    {:else}
                      {formatValue(cellValue)}
                    {/if}
                  </td>
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>

        {#if isLoadingMore}
          <div class="text-center py-3 bg-light">
            <i class="fas fa-spinner fa-spin text-primary"></i>
            <span class="ms-2 text-muted">Loading more data...</span>
          </div>
        {/if}

        {#if !hasMoreData && displayRows.length > 0}
          <div class="text-center py-3 text-muted small bg-light">
            <i class="fas fa-check-circle"></i>
            <span class="ms-2"
              >All data loaded ({displayRows.length.toLocaleString()} rows)</span
            >
          </div>
        {/if}
      </div>
    {:else}
      <!-- JSON View -->
      <div class="json-container flex-grow-1 p-3" bind:this={tableWrapper}>
        <div class="json-list">
          {#each displayRows as row, index}
            <div class="json-item">
              <div class="json-item-header">
                <span class="json-item-number">#{index + 1}</span>
              </div>
              <pre class="json-content">{stringifyRowWithOrder(
                  row,
                  displayData.columns
                )}</pre>
            </div>
          {/each}
        </div>

        {#if isLoadingMore}
          <div class="text-center py-3 bg-light">
            <i class="fas fa-spinner fa-spin text-primary"></i>
            <span class="ms-2 text-muted">Loading more data...</span>
          </div>
        {/if}

        {#if !hasMoreData && displayRows.length > 0}
          <div class="text-center py-3 text-muted small bg-light">
            <i class="fas fa-check-circle"></i>
            <span class="ms-2"
              >All data loaded ({displayRows.length.toLocaleString()} rows)</span
            >
          </div>
        {/if}
      </div>
    {/if}
  {:else if displayData}
    <div
      class="d-flex flex-column align-items-center justify-content-center h-100 text-secondary"
    >
      <i class="fas fa-info-circle fa-3x mb-3 opacity-50"></i>
      <p class="fs-5">Query executed successfully</p>
      {#if displayData.rows_affected !== null}
        <span class="badge bg-success"
          >{displayData.rows_affected} rows affected</span
        >
      {/if}
    </div>
  {:else}
    <div
      class="d-flex flex-column align-items-center justify-content-center h-100 text-secondary"
    >
      <i class="fas fa-table fa-3x mb-3 opacity-25"></i>
      <p class="fs-5">No data to display</p>
      <p class="text-muted">Execute a query to see results here</p>
    </div>
  {/if}

  <!-- Sticky footer untuk Save/Cancel buttons -->
  {#if hasUnsavedEdits}
    <div
      class="sticky-bottom bg-warning border-top shadow-sm"
      style="position: sticky; bottom: 0; z-index: 11;"
    >
      <div class="d-flex align-items-center justify-content-between gap-2 p-2">
        <div class="d-flex align-items-center gap-2">
          <i class="fas fa-exclamation-triangle text-danger"></i>
          <span class="fw-semibold text-danger">
            You have unsaved changes ({editedRows.size} row{editedRows.size > 1
              ? "s"
              : ""} modified)
          </span>
        </div>
        <div class="d-flex gap-2">
          <button class="btn btn-sm btn-success" on:click={showPreview}>
            <i class="fas fa-save"></i> Save Changes
          </button>
          <button class="btn btn-sm btn-secondary" on:click={cancelAllEdits}>
            <i class="fas fa-times"></i> Cancel
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Sticky footer untuk menampilkan final query -->
  {#if finalQuery || executedQuery}
    <div
      class="sticky-bottom bg-light border-top shadow-sm"
      style="position: sticky; bottom: 0; z-index: 10;"
    >
      <div class="d-flex align-items-center gap-2 p-2 font-monospace small">
        <div
          class="d-flex align-items-center gap-2 text-primary fw-semibold"
          style="min-width: 80px;"
        >
          <i class="fas fa-code"></i>
          <span>Query:</span>
        </div>
        <div
          class="flex-grow-1 text-truncate bg-white px-2 py-1 border rounded"
          title={finalQuery || executedQuery}
        >
          {finalQuery || executedQuery}
        </div>
      </div>
    </div>
  {/if}
</div>

<!-- SQL Preview Modal -->
{#if showSqlPreview}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal-backdrop show" on:click={closeSqlPreview}></div>
  <div class="modal d-block" tabindex="-1">
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      class="modal-dialog modal-lg modal-dialog-centered"
      on:click|stopPropagation
    >
      <div class="modal-content">
        <div class="modal-header bg-primary text-white">
          <h5 class="modal-title">
            <i class="fas fa-code"></i> Preview SQL Update
          </h5>
          <button
            type="button"
            class="btn-close btn-close-white"
            on:click={closeSqlPreview}
            aria-label="Close"
          ></button>
        </div>

        <div class="modal-body">
          <div class="alert alert-info">
            <i class="fas fa-info-circle"></i>
            <strong>{pendingUpdates.length}</strong> update(s) will be executed:
          </div>

          <pre
            class="bg-dark text-light p-3 rounded"
            style="max-height: 400px; overflow-y: auto;"><code
              >{previewSql}</code
            ></pre>

          <div class="alert alert-warning mt-3">
            <i class="fas fa-exclamation-triangle"></i>
            <strong>Warning:</strong> This action cannot be undone. Please review
            the SQL carefully before executing.
          </div>
        </div>

        <div class="modal-footer">
          <button
            type="button"
            class="btn btn-secondary"
            on:click={closeSqlPreview}
          >
            <i class="fas fa-times"></i> Cancel
          </button>
          <button
            type="button"
            class="btn btn-success"
            on:click={executeUpdates}
          >
            <i class="fas fa-play"></i> Execute
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

{#if showFilterModal && filterModalColumn}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal-backdrop show" on:click={closeFilterModal}></div>
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal d-block" tabindex="-1" on:click={closeFilterModal}>
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      class="modal-dialog"
      style="position: fixed; top: {filterModalPosition.top}px; left: {filterModalPosition.left}px; margin: 0;"
      on:click|stopPropagation
    >
      <div class="modal-content">
        <div class="modal-header">
          <h5 class="modal-title">
            <i class="fas fa-filter"></i> Filter: {filterModalColumn}
          </h5>
          <button
            type="button"
            class="btn-close"
            on:click={closeFilterModal}
            aria-label="Close"
          ></button>
        </div>

        <div class="modal-body">
          <input
            type="text"
            class="form-control mb-3"
            placeholder="Search values..."
            bind:value={filterSearchQuery}
          />

          <div class="d-flex gap-2 mb-2">
            <button
              class="btn btn-sm btn-outline-primary flex-fill"
              on:click={selectAllFilterValues}
            >
              <i class="fas fa-check-double"></i> Select All
            </button>
            <button
              class="btn btn-sm btn-outline-secondary flex-fill"
              on:click={deselectAllFilterValues}
            >
              <i class="fas fa-times"></i> Deselect All
            </button>
          </div>

          <div
            class="border rounded"
            style="max-height: 300px; overflow-y: auto;"
          >
            {#if loadingFilterValues}
              <div
                class="d-flex flex-column align-items-center justify-content-center p-4 text-primary"
              >
                <i class="fas fa-spinner fa-spin fa-2x mb-2"></i>
                <span>Loading values...</span>
              </div>
            {:else}
              {@const cacheKey = `${filterModalColumn}_${filterSearchQuery || ""}`}
              {@const availableValues =
                filterValuesCache[cacheKey] ||
                getDistinctValues(filterModalColumn)}
              <table class="table table-sm table-hover mb-0">
                <tbody>
                  {#each availableValues as value}
                    <tr>
                      <td class="text-center" style="width: 40px;">
                        <input
                          class="form-check-input"
                          type="checkbox"
                          id="filter-{value}"
                          checked={selectedFilterValues[filterModalColumn]?.has(
                            value
                          ) || false}
                          on:change={() =>
                            toggleFilterValue(filterModalColumn, value)}
                        />
                      </td>
                      <td>
                        <label
                          class="form-check-label w-100 mb-0"
                          for="filter-{value}"
                          title={value}
                          style="cursor: pointer;"
                        >
                          {value}
                        </label>
                      </td>
                    </tr>
                  {:else}
                    <tr>
                      <td colspan="2" class="text-center p-4 text-muted">
                        <i class="fas fa-info-circle fa-2x mb-2"></i>
                        <div>No values found</div>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {/if}
          </div>
        </div>

        <div class="modal-footer">
          <button class="btn btn-primary" on:click={applyColumnFilter}>
            <i class="fas fa-check"></i> Apply
          </button>
          <button class="btn btn-danger" on:click={clearColumnFilter}>
            <i class="fas fa-eraser"></i> Clear
          </button>
          <button class="btn btn-secondary" on:click={closeFilterModal}>
            Cancel
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<!-- Popup Editor Modal for Long Content -->
{#if showPopupEditor && popupEditingCell}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal-backdrop show" on:click={closePopupEditor}></div>
  <div class="modal d-block" tabindex="-1">
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      class="modal-dialog modal-lg modal-dialog-centered"
      on:click|stopPropagation
    >
      <div class="modal-content">
        <div class="modal-header bg-primary text-white">
          <h5 class="modal-title">
            <i class="fas fa-edit"></i> Edit Cell Value
          </h5>
          <button
            type="button"
            class="btn-close btn-close-white"
            on:click={closePopupEditor}
            aria-label="Close"
          ></button>
        </div>

        <div class="modal-body">
          <div class="mb-2">
            <strong>Column:</strong>
            <span class="badge bg-info">{popupEditingCell.column}</span>
            <strong class="ms-3">Row:</strong>
            <span class="badge bg-info">{popupEditingCell.rowIndex + 1}</span>
          </div>
          <textarea
            class="form-control font-monospace"
            rows="15"
            bind:value={popupEditorValue}
            placeholder="Enter value..."
            style="resize: vertical; min-height: 200px;"
            on:keydown={(e) => {
              if (e.key === "Escape") {
                closePopupEditor();
              } else if (e.key === "Enter" && e.ctrlKey) {
                e.preventDefault();
                savePopupEdit();
              }
            }}
            use:focusInput
          ></textarea>
          <div class="form-text mt-2">
            <i class="fas fa-info-circle"></i> Press Ctrl+Enter to save, Escape to
            cancel
          </div>
        </div>

        <div class="modal-footer">
          <button
            type="button"
            class="btn btn-secondary"
            on:click={closePopupEditor}
          >
            <i class="fas fa-times"></i> Cancel
          </button>
          <button
            type="button"
            class="btn btn-primary"
            on:click={savePopupEdit}
          >
            <i class="fas fa-check"></i> Save
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  /* Custom scrollbar styling */
  .table-container::-webkit-scrollbar {
    width: 12px;
    height: 12px;
  }

  .table-container::-webkit-scrollbar-track {
    background: #f8f9fa;
  }

  .table-container::-webkit-scrollbar-thumb {
    background: #c0c0c0;
    border-radius: 6px;
  }

  .table-container::-webkit-scrollbar-thumb:hover {
    background: #a0a0a0;
  }

  /* Null value styling */
  .null-value {
    color: #6c757d;
  }

  /* Column header styling */
  .column-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.25rem;
  }

  .sort-button {
    flex: 1;
    border: none;
    background: none;
    padding: 0.25rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.25rem;
    color: inherit;
  }

  .sort-button:hover {
    color: #0d6efd;
  }

  .sort-icon {
    font-size: 0.75rem;
  }

  .sort-icon.inactive {
    opacity: 0.3;
  }

  .filter-icon-button {
    border: none;
    background: none;
    padding: 0.25rem 0.5rem;
    cursor: pointer;
    color: #6c757d;
    font-size: 0.875rem;
  }

  .filter-icon-button:hover {
    color: #0d6efd;
  }

  .filter-icon-button.active {
    color: #0d6efd;
  }

  .column-name {
    font-weight: 600;
  }

  /* Table container */
  .table-container {
    position: relative;
    overflow: auto;
    height: 100%;
  }

  /* Optimize table rendering */
  .data-table {
    border-collapse: collapse;
    table-layout: auto;
    margin: 0;
    width: auto;
    min-width: 100%;
  }

  .data-table thead th {
    position: relative;
    background-color: #e7f1ff;
  }

  /* Ensure table cells truncate properly */
  .data-table td,
  .data-table th {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 0.5rem;
    border: 1px solid #dee2e6;
    max-width: 500px;
    min-width: 100px;
  }

  /* Fixed row height to prevent layout shifts */
  .data-table tbody tr {
    height: 32px;
  }

  .data-table tbody td {
    height: 32px;
    line-height: 1.2;
  }

  /* Inline editing styles */
  .data-table tbody td {
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .data-table tbody td:hover {
    background-color: #f8f9fa;
  }

  .data-table tbody td.editing {
    padding: 2px;
    background-color: #fff3cd;
  }

  .data-table tbody td.editing input {
    width: 100%;
    height: 100%;
    border: 2px solid #ffc107;
    padding: 0.25rem;
  }

  .data-table tbody tr.edited-row {
    background-color: #f8d7da;
  }

  .data-table tbody tr.edited-row:hover {
    background-color: #f1aeb5;
  }

  .data-table tbody tr.edited-row td {
    color: #842029;
  }

  .data-table tbody td.edited-cell {
    background-color: #f8d7da;
    color: #842029;
    border: 2px solid #dc3545;
    font-weight: 500;
  }

  .data-table tbody td.edited-cell:hover {
    background-color: #f1aeb5;
  }

  /* Disable all animations and transitions */
  .data-table,
  .data-table *,
  .table-container,
  .table-container * {
    transition: none !important;
    animation: none !important;
  }

  /* Prevent layout shift during scroll */
  .data-table tbody {
    display: table-row-group;
  }

  /* JSON View Styles */
  .json-container {
    position: relative;
    overflow: auto;
    height: 100%;
    background-color: #f8f9fa;
  }

  .json-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .json-item {
    background: white;
    border: 1px solid #dee2e6;
    border-radius: 0.375rem;
    overflow: hidden;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .json-item-header {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    padding: 0.5rem 1rem;
    font-weight: 600;
    font-size: 0.875rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .json-item-number {
    background: rgba(255, 255, 255, 0.2);
    padding: 0.125rem 0.5rem;
    border-radius: 0.25rem;
    font-family: monospace;
  }

  .json-content {
    margin: 0;
    padding: 1rem;
    background-color: #ffffff;
    font-family: "Consolas", "Monaco", "Courier New", monospace;
    font-size: 0.875rem;
    line-height: 1.5;
    overflow-x: auto;
    color: #2d3748;
    border: none;
  }

  .json-content::-webkit-scrollbar {
    height: 8px;
  }

  .json-content::-webkit-scrollbar-track {
    background: #f1f1f1;
  }

  .json-content::-webkit-scrollbar-thumb {
    background: #c0c0c0;
    border-radius: 4px;
  }

  .json-content::-webkit-scrollbar-thumb:hover {
    background: #a0a0a0;
  }

  /* Array cell styling */
  .array-cell-td {
    vertical-align: top;
    min-width: 150px;
  }
</style>
