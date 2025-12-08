<script>
  import { onMount, afterUpdate, tick } from "svelte";
  import { tabDataStore } from "../../../shared/stores/tabData";
  import { defaultPaginateLimit } from "../../../shared/stores/appSettings";
  import { DatabaseType } from "../../../core/config/databaseTypes";

  // Views
  import GridView from "./views/GridView.svelte";
  import JsonView from "./views/JsonView.svelte";

  // Partials
  import DataGridHeader from "./partials/DataGridHeader.svelte";
  import DataGridFooter from "./partials/DataGridFooter.svelte"; // Modals
  import FilterModal from "../modals/FilterModal.svelte";
  import CellEditorModal from "../modals/CellEditorModal.svelte";

  // Services
  import {
    loadTableInitial,
    loadQueryInitial,
    appendTableData,
    appendQueryData,
    loadFilterValuesFromServer,
  } from "../services/dataGridService";
  import {
    startEdit,
    cancelAllEdits as cancelAllEditsService,
  } from "../services/gridEditService";
  import { openFilterModal as openFilterModalService } from "../services/gridFilterService";
  import { handleSort } from "../services/gridSortService";

  // Utils
  import {
    syncColumnWidths,
    isScrolledNearBottom,
    isScrollingDown,
    shouldLoadMore,
  } from "../../../shared/utils/grid/gridScrollSync";
  import { getDistinctValues } from "../../../shared/utils/data/dataHelpers";

  export let tabId = null;
  export let executedQuery = "";
  export let connection = null;
  export let tableName = "";
  export let databaseName = "";
  export let schemaName = "";
  export let isTableMode = true; // true = load by tableName, false = load by query

  // State
  let displayData = null;
  let isFiltered = false;
  let totalRows = 0;
  let paginateLimit; // Will be set from store or default to 200
  let currentOffset = 0;
  let isLoadingMore = false;
  let hasMoreData = true;
  let lastLoadedQueryId = null;
  let columnFilters = {};
  let sortStack = [];
  let showFilterModal = false;
  let filterModalColumn = null;
  let filterModalPosition = { top: 0, left: 0 };
  let filterSearchQuery = "";
  let selectedFilterValues = {};
  let filterValuesCache = {};
  let loadingFilterValues = false;
  let isLoadingData = false;
  let finalQuery = "";
  let currentTabId = null;
  let isRestoringScroll = false;
  let viewMode = "grid";
  let lastDisplayDataSource = null; // Track whether data came from sort/filter or parent prop

  // Editing state
  let editingCell = null;
  let editingValue = "";
  let originalValue = "";
  let editedRows = new Map();
  let newRows = new Map();
  let deletedRows = new Set();
  let originalRowData = new Map();
  let showPopupEditor = false;
  let popupEditorValue = "";
  let popupEditingCell = null;
  let selectedCell = null;
  let selectedRows = new Set();
  let displayRows = [];

  // Performance timing
  let executionTime = 0; // Total time from request start to data render (ms)
  let fetchTime = 0; // Only BE fetch time (ms)

  // Auto-refresh state (managed at parent level to survive component remounting)
  let autoRefreshInterval = null;
  let isAutoRefreshActive = false;

  // Scroll state
  let tableWrapper;
  let headerWrapper;
  let rowNumbersWrapper;
  let scrollTimeout;
  let lastScrollTop = 0;
  let lastLoadTriggeredAt = 0;

  // Display names and columns
  $: displayNames =
    displayData?.columns?.map((col) =>
      typeof col === "string" ? col : col.name
    ) || [];

  $: columnNames =
    displayData?.columns?.map((col) =>
      typeof col === "string" ? col : col.name
    ) || [];

  $: if (displayData?.rows && displayData.rows.length > 0) {
    // Only update displayRows if it comes from displayData (initial load or refresh)
    // Don't update if we're in the middle of editing
    if (!editingCell && !showPopupEditor) {
      console.log(
        "[DataGrid] Reactive: Updating displayRows from displayData",
        {
          displayDataLength: displayData.rows.length,
          previousDisplayRowsLength: displayRows.length,
        }
      );
      displayRows = displayData.rows;
    }
  }

  // Set default view mode
  $: if (connection && !$tabDataStore[tabId]?.viewMode) {
    viewMode = connection.db_type === DatabaseType.MONGODB ? "json" : "grid";
  }

  // Load saved state when tab changes
  $: if (tabId && tabId !== currentTabId) {
    currentTabId = tabId;
    console.log(
      `[DataGrid] Tab changed to ${tabId}, resetting paginateLimit to 200`
    );

    // Reset paginateLimit to default when switching tabs
    // Force a new reactive update by creating a new object reference
    paginateLimit = 200;
    // Trigger a manual re-render to ensure DataGridFooter gets updated value
    Promise.resolve().then(() => {
      // Force parent to update bindings
    });

    if ($tabDataStore[tabId]) {
      const savedState = $tabDataStore[tabId];
      columnFilters = savedState.filters || {};
      sortStack = savedState.sortStack || [];
      viewMode =
        savedState.viewMode ||
        (connection?.db_type === DatabaseType.MONGODB ? "json" : "grid");

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

  // Initialize paginateLimit from store
  $: if (!paginateLimit) {
    paginateLimit = $defaultPaginateLimit || 200;
  }

  // Auto-load data when table or query changes
  $: if (
    (tableName || executedQuery) &&
    connection &&
    !isLoadingData &&
    !isLoadingMore
  ) {
    const queryId = tableName
      ? tableName + JSON.stringify(columnFilters) + JSON.stringify(sortStack)
      : executedQuery +
        JSON.stringify(columnFilters) +
        JSON.stringify(sortStack);

    // Only load if this is a new table/query or if filters/sorts changed
    const isInitialLoad =
      sortStack.length === 0 && Object.keys(columnFilters).length === 0;

    if (isInitialLoad && queryId !== lastLoadedQueryId) {
      handleReloadData();
    }
  }

  // Clear cache when query changes
  $: if (executedQuery) {
    filterValuesCache = {};
    columnFilters = {};
    sortStack = [];
    currentOffset = 0;
    hasMoreData = true;
    lastLoadedQueryId = null;
  }

  afterUpdate(() => {
    if (viewMode === "grid" && displayData?.rows?.length > 0) {
      syncColumnWidths(headerWrapper, tableWrapper);
    }
  });

  let resizeObserver;
  onMount(() => {
    resizeObserver = new ResizeObserver(() => {
      if (viewMode === "grid" && displayData?.rows?.length > 0) {
        syncColumnWidths(headerWrapper, tableWrapper);
      }
    });

    // Listen for reload-datagrid event from menuHandlers
    const handleReloadEvent = (e) => {
      if (e.detail?.tabId === tabId) {
        handleReloadData();
      }
    };
    document.addEventListener("reload-datagrid", handleReloadEvent);

    return () => {
      if (resizeObserver) {
        resizeObserver.disconnect();
      }
      document.removeEventListener("reload-datagrid", handleReloadEvent);
      // Cleanup auto-refresh when component is destroyed
      stopAutoRefresh();
    };
  });

  $: if (tableWrapper && resizeObserver) {
    resizeObserver.observe(tableWrapper);
  }

  // Auto-refresh management function
  function setupAutoRefresh(intervalMs) {
    // Clear any existing interval
    if (autoRefreshInterval) {
      clearInterval(autoRefreshInterval);
      autoRefreshInterval = null;
    }

    if (intervalMs <= 0) {
      isAutoRefreshActive = false;
      return;
    }

    isAutoRefreshActive = true;
    autoRefreshInterval = setInterval(() => {
      handleReloadData();
    }, intervalMs);
    console.log(`🔄 Auto-refresh setup: every ${intervalMs}ms`);
  }

  function stopAutoRefresh() {
    if (autoRefreshInterval) {
      clearInterval(autoRefreshInterval);
      autoRefreshInterval = null;
      isAutoRefreshActive = false;
      console.log("⏹️ Auto-refresh stopped");
    }
  }

  async function handleReloadData() {
    const previousData = displayData;
    displayData = null;
    isLoadingData = true;
    isFiltered = Object.keys(columnFilters).length > 0 || sortStack.length > 0;
    currentOffset = 0;
    hasMoreData = true;

    // Start timing from request start
    const totalStartTime = performance.now();
    let fetchStartTime = 0;
    let fetchEndTime = 0;

    await new Promise((resolve) => setTimeout(resolve, 0));

    try {
      let result;

      // If this is a table query (from sidebar), use loadTableInitial
      if (isTableMode && tableName) {
        fetchStartTime = performance.now();
        result = await loadTableInitial(
          connection,
          tableName,
          databaseName,
          schemaName,
          columnFilters,
          sortStack,
          paginateLimit
        );
        fetchEndTime = performance.now();
        lastLoadedQueryId =
          tableName + JSON.stringify(columnFilters) + JSON.stringify(sortStack);
      }
      // If this is a custom query (from SQL editor), use loadQueryInitial
      else if (!isTableMode && executedQuery && executedQuery.trim() !== "") {
        fetchStartTime = performance.now();
        result = await loadQueryInitial(
          connection,
          executedQuery,
          tableName,
          databaseName,
          columnFilters,
          sortStack,
          paginateLimit
        );
        fetchEndTime = performance.now();
        lastLoadedQueryId =
          executedQuery +
          JSON.stringify(columnFilters) +
          JSON.stringify(sortStack);
      } else {
        // No valid query or table
        displayData = previousData;
        isLoadingData = false;
        return;
      }

      // Calculate fetch time (BE request time only)
      fetchTime = Math.round(fetchEndTime - fetchStartTime);

      displayData = result;
      totalRows = result?.total_count || result?.rows?.length || 0;
      currentOffset = result?.rows?.length || 0;
      // Check if there's more data by comparing with paginateLimit
      hasMoreData = result?.rows?.length === paginateLimit;

      // Explicitly update displayRows with new data
      console.log(`[DataGrid] Setting displayRows explicitly:`, {
        newLength: result?.rows?.length || 0,
        paginateLimit,
      });
      displayRows = result?.rows || [];

      if (result.final_query) {
        finalQuery = result.final_query;
      }

      // Auto-select first cell (column 0, row 0) when table loads
      if (displayRows.length > 0 && result?.columns?.length > 0) {
        const firstColumnName =
          typeof result.columns[0] === "string"
            ? result.columns[0]
            : result.columns[0].name;
        const firstRowValue = Array.isArray(displayRows[0])
          ? displayRows[0][0]
          : displayRows[0][firstColumnName];
        selectedCell = {
          rowIndex: 0,
          column: firstColumnName,
          currentValue: firstRowValue,
        };
        selectedRows.clear();
        console.log(
          `[DataGrid] Auto-selected cell at row 0, column 0:`,
          selectedCell
        );
      }

      // Mark that this data came from our sort/filter operation, not from parent prop
      lastDisplayDataSource = "sort-filter";

      // Update tabDataStore so parent component has the latest sorted data
      if (tabId) {
        tabDataStore.setQueryResult(tabId, result);
      }

      // Debug: Log the data received
      console.log(`[DataGrid] Data loaded and set to displayData:`, {
        sortStack,
        final_query: result?.final_query,
        total_rows: totalRows,
        first_row: result?.rows?.[0],
        last_row: result?.rows?.[result?.rows?.length - 1],
        displayRows_length: result?.rows?.length,
      });
    } catch (error) {
      console.error("❌ Failed to reload data:", error);
      displayData = previousData;
      isFiltered = false;
    } finally {
      isLoadingData = false;
      // Calculate total execution time (from request start to render complete)
      const totalEndTime = performance.now();
      executionTime = Math.round(totalEndTime - totalStartTime);
      console.log(
        `[DataGrid] Execution time: ${executionTime}ms, Fetch time: ${fetchTime}ms`
      );
    }
  }

  async function handleLoadMore() {
    if (isLoadingMore || !hasMoreData || !connection) {
      return;
    }

    // Skip if no table name and no executed query
    if (!tableName && (!executedQuery || executedQuery.trim() === "")) {
      return;
    }

    isLoadingMore = true;
    const totalStartTime = performance.now();
    let fetchStartTime = 0;
    let fetchEndTime = 0;

    try {
      let result;

      // Use appropriate append function based on data source
      if (isTableMode && tableName) {
        fetchStartTime = performance.now();
        result = await appendTableData(
          connection,
          tableName,
          databaseName,
          schemaName,
          columnFilters,
          sortStack,
          currentOffset,
          paginateLimit
        );
        fetchEndTime = performance.now();
      } else if (!isTableMode && executedQuery && executedQuery.trim() !== "") {
        fetchStartTime = performance.now();
        result = await appendQueryData(
          connection,
          executedQuery,
          tableName,
          databaseName,
          columnFilters,
          sortStack,
          currentOffset,
          paginateLimit
        );
        fetchEndTime = performance.now();
      } else {
        isLoadingMore = false;
        return;
      }

      // Calculate fetch time
      fetchTime = Math.round(fetchEndTime - fetchStartTime);

      if (result?.rows && displayData) {
        displayData.rows = [...displayData.rows, ...result.rows];
        currentOffset += result.rows.length;
        // Check if more data is available based on paginateLimit
        hasMoreData = result?.rows?.length === paginateLimit;
      }
    } catch (error) {
      console.error("❌ Failed to load more data:", error);
      hasMoreData = false;
    } finally {
      isLoadingMore = false;
      // Calculate total execution time
      const totalEndTime = performance.now();
      executionTime = Math.round(totalEndTime - totalStartTime);
      console.log(
        `[DataGrid] Load more - Execution time: ${executionTime}ms, Fetch time: ${fetchTime}ms`
      );
    }
  }

  // Scroll handling
  function handleGridScroll({ detail }) {
    if (!tableWrapper || isRestoringScroll) return;

    const { scrollTop, scrollLeft } = detail;

    if (headerWrapper) {
      headerWrapper.scrollLeft = scrollLeft;
    }

    if (rowNumbersWrapper) {
      rowNumbersWrapper.scrollTop = scrollTop;
    }

    const now = Date.now();
    const isScrolling = isScrollingDown(scrollTop, lastScrollTop);
    const canTriggerLoad = now - lastLoadTriggeredAt > 1000;

    if (
      shouldLoadMore(
        isScrolledNearBottom(tableWrapper, 200),
        hasMoreData,
        isLoadingMore,
        isScrolling,
        canTriggerLoad ? 1 : 0,
        1000
      )
    ) {
      lastLoadTriggeredAt = now;
      handleLoadMore();
    }

    if (Math.abs(scrollTop - lastScrollTop) >= 5) {
      lastScrollTop = scrollTop;

      if (scrollTimeout) {
        clearTimeout(scrollTimeout);
      }

      scrollTimeout = setTimeout(() => {
        if (tabId && tableWrapper) {
          tabDataStore.setScrollPosition(tabId, tableWrapper.scrollTop);
        }
      }, 150);
    }
  }

  // Sorting
  function handleSortClick(column, event) {
    const isCtrlPressed = event?.ctrlKey || event?.metaKey || false;
    const isShiftPressed = event?.shiftKey || false;

    console.log(
      `🔤 Sort click on "${column}" - Ctrl: ${isCtrlPressed}, Shift: ${isShiftPressed}, Current stack:`,
      sortStack
    );

    const result = handleSort(
      column,
      sortStack,
      tabId,
      tabDataStore,
      handleReloadData,
      isCtrlPressed,
      isShiftPressed
    );
    console.log(`📊 New sort stack:`, result.sortStack);
    sortStack = result.sortStack;
  }

  // Filtering
  function handleFilterClick(column, event) {
    console.log("🟢 handleFilterClick called for column:", column);
    const result = openFilterModalService(
      column,
      event,
      columnFilters,
      selectedFilterValues
    );
    if (result) {
      filterModalColumn = result.column;
      filterModalPosition = result.position;
      filterSearchQuery = "";
      showFilterModal = true;
      console.log(
        "🟢 showFilterModal set to true, filterModalColumn:",
        filterModalColumn
      );
      handleLoadFilterValues(column);
    }
  }

  async function handleLoadFilterValues(column, search = null) {
    const cacheKey = `${column}_${search || ""}`;

    if (filterValuesCache[cacheKey]) {
      return;
    }

    if (!executedQuery || executedQuery.trim() === "" || !connection) {
      filterValuesCache[cacheKey] = getDistinctValues(displayData, column);
      filterValuesCache = { ...filterValuesCache };
      return;
    }

    loadingFilterValues = true;

    try {
      const result = await loadFilterValuesFromServer(
        connection,
        tableName,
        databaseName,
        schemaName,
        column,
        search
      );

      filterValuesCache[cacheKey] = result;
      filterValuesCache = { ...filterValuesCache };
    } catch (error) {
      console.error("❌ Failed to load filter values:", error);
      filterValuesCache[cacheKey] = getDistinctValues(displayData, column);
      filterValuesCache = { ...filterValuesCache };
    } finally {
      loadingFilterValues = false;
    }
  }

  function handleApplyFilter(event) {
    console.log(
      "🟢 handleApplyFilter called, column:",
      event?.detail?.column || filterModalColumn
    );
    const column = event?.detail?.column || filterModalColumn;
    const selected =
      event?.detail?.selectedValues || selectedFilterValues[column];

    if (!column) return;

    if (selected && selected.size > 0) {
      columnFilters[column] = Array.from(selected);
    } else {
      delete columnFilters[column];
    }

    columnFilters = { ...columnFilters };

    if (tabId) {
      tabDataStore.setFilters(tabId, columnFilters);
    }

    // Use tick() to defer closing until event is fully processed
    tick().then(() => {
      console.log("🟢 After tick, calling closeFilterModal");
      closeFilterModal();
      handleReloadData();
    });
  }

  function handleClearFilter(event) {
    const column = event?.detail?.column || filterModalColumn;

    if (!column) return;

    delete columnFilters[column];
    delete selectedFilterValues[column];
    columnFilters = { ...columnFilters };
    selectedFilterValues = { ...selectedFilterValues };

    if (tabId) {
      tabDataStore.setFilters(tabId, columnFilters);
    }

    finalQuery = "";

    // Use tick() to defer closing until event is fully processed
    tick().then(() => {
      closeFilterModal();
      handleReloadData();
    });
  }

  function closeFilterModal() {
    console.log(
      "🟢 closeFilterModal called, setting showFilterModal=false, filterModalColumn=null"
    );
    showFilterModal = false;
    filterModalColumn = null;
    filterSearchQuery = "";
  }

  function handleClearAllFilters() {
    columnFilters = {};
    selectedFilterValues = {};

    if (tabId) {
      tabDataStore.setFilters(tabId, columnFilters);
    }

    finalQuery = "";
    handleReloadData();
  }

  // Inline filter input - triggered on Enter key press
  function handleFilterInput(column, value) {
    // Don't allow typing if array filter is active
    if (Array.isArray(columnFilters[column])) {
      return;
    }

    if (value && value.trim() !== "") {
      columnFilters[column] = value.trim();
    } else {
      delete columnFilters[column];
    }

    columnFilters = { ...columnFilters };

    if (tabId) {
      tabDataStore.setFilters(tabId, columnFilters);
    }

    handleReloadData();
  }

  // Editing
  function handleCellClick(rowIndex, column, currentValue) {
    // Toggle: if same cell is clicked again, unselect it
    if (
      selectedCell?.rowIndex === rowIndex &&
      selectedCell?.column === column
    ) {
      selectedCell = null;
    } else {
      selectedCell = { rowIndex, column, currentValue };
    }

    // Clear row selection when cell is selected (mutually exclusive)
    selectedRows.clear();
    selectedRows = new Set(selectedRows); // Trigger reactivity
  }

  function handleRowNumberClick(rowIndex) {
    // Toggle row selection
    if (selectedRows.has(rowIndex)) {
      selectedRows.delete(rowIndex);
    } else {
      selectedRows.clear(); // Clear previous selection for single select
      selectedRows.add(rowIndex);
    }
    selectedRows = new Set(selectedRows); // Trigger reactivity

    // Clear cell selection when row is selected (mutually exclusive)
    selectedCell = null;
  }

  function handleEditCellButton() {
    // Trigger edit on selected cell from toolbar button
    if (!selectedCell) {
      console.warn("⚠️ Please select a cell to edit");
      return;
    }

    const { rowIndex, column, currentValue } = selectedCell;

    handleCellDoubleClick(rowIndex, column, currentValue);
  }

  function handleCellDoubleClick(rowIndex, column, currentValue) {
    const result = startEdit(rowIndex, column, currentValue);

    if (result.shouldUsePopupEditor) {
      showPopupEditor = true;
      popupEditorValue = result.valueStr;
      popupEditingCell = { rowIndex, column };
      originalValue = result.originalValue;
    } else {
      editingCell = { rowIndex, column };
      editingValue = result.valueStr;
      originalValue = result.originalValue;
    }
  }

  function handleCellKeydown(rowIndex, column, event) {
    if (event.key === "Escape") {
      event.preventDefault();
      cancelEdit();
    } else if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      handleCellBlur(rowIndex, column, event);
    }
  }

  function isValidForColumnType(value, column) {
    // Find column metadata
    if (!displayData.columns) return true;

    const columnMeta = displayData.columns.find((col) => col.name === column);
    if (!columnMeta) return true;

    const dataType = (columnMeta.data_type || "").toUpperCase();

    // Handle null/empty - always valid
    if (value === "" || value === null) {
      return true;
    }

    // Validate based on data type
    if (
      dataType.includes("INT") ||
      dataType.includes("DECIMAL") ||
      dataType.includes("NUMERIC") ||
      dataType.includes("FLOAT") ||
      dataType.includes("DOUBLE") ||
      dataType.includes("NUMBER") ||
      dataType.includes("REAL")
    ) {
      // Numeric type - must be valid number
      const num = Number(value);
      return !isNaN(num) && value.trim() !== "";
    } else if (dataType.includes("BOOL")) {
      // Boolean type - allow true/false/0/1
      return ["true", "false", "0", "1", "yes", "no"].includes(
        value.toLowerCase()
      );
    } else {
      // String type - always valid
      return true;
    }
  }

  function convertValueToColumnType(value, column) {
    // Find column metadata
    if (!displayData.columns) return value;

    const columnMeta = displayData.columns.find((col) => col.name === column);
    if (!columnMeta) return value;

    const dataType = (columnMeta.data_type || "").toUpperCase();

    // Handle null/empty
    if (value === "" || value === null) {
      return null;
    }

    // Convert based on data type
    if (
      dataType.includes("INT") ||
      dataType.includes("DECIMAL") ||
      dataType.includes("NUMERIC") ||
      dataType.includes("FLOAT") ||
      dataType.includes("DOUBLE") ||
      dataType.includes("NUMBER") ||
      dataType.includes("REAL")
    ) {
      // Numeric type
      const num = Number(value);
      return isNaN(num) ? value : num;
    } else if (dataType.includes("BOOL")) {
      // Boolean type
      return value === "true" || value === "1" || value === true;
    } else if (dataType.includes("DATE") || dataType.includes("TIME")) {
      // Keep as string for dates
      return value;
    } else {
      // String type
      return value;
    }
  }

  function handleCellBlur(rowIndex, column, event) {
    if (!editingCell) return;

    // Check if value has changed - handle null/undefined properly
    const originalStr =
      originalValue === null || originalValue === undefined
        ? ""
        : String(originalValue);
    const hasChanged = editingValue !== originalStr;

    if (hasChanged) {
      // VALIDATE: Check if new value is valid for column type
      if (!isValidForColumnType(editingValue, column)) {
        console.warn(
          `[handleCellBlur] ❌ Invalid value for column ${column}: "${editingValue}"`
        );
        // Reset to original value
        displayRows[rowIndex] = Array.isArray(displayRows[rowIndex])
          ? [...displayRows[rowIndex]]
          : { ...displayRows[rowIndex] };
        displayRows = displayRows;
        editingValue = originalStr;
        cancelEdit();
        alert(
          `❌ Invalid value for ${column}. Please enter a valid ${displayData.columns?.find((c) => c.name === column)?.data_type || "value"}.`
        );
        return;
      }
      // IMPORTANT: Backup original data BEFORE updating displayData
      // (first time edit on this row)
      if (!originalRowData.has(rowIndex)) {
        const rowToBackup = displayData.rows[rowIndex];
        let colNames = displayData.column_names || [];
        if (colNames.length === 0 && displayData.columns) {
          colNames = displayData.columns.map((col) => col.name);
        }

        // Convert array to object if needed
        const backupData = Array.isArray(rowToBackup)
          ? Object.fromEntries(
              colNames.map((name, idx) => [name, rowToBackup[idx]])
            )
          : { ...rowToBackup };

        originalRowData.set(rowIndex, backupData);
        console.log(
          `[handleCellBlur] Backed up original row ${rowIndex}:`,
          backupData
        );
      }

      // Update displayRows with the new value
      // column is a string (column name), not an object
      const columnIndex = columnNames.indexOf(column);

      const newRow = Array.isArray(displayRows[rowIndex])
        ? [...displayRows[rowIndex]]
        : { ...displayRows[rowIndex] };

      if (Array.isArray(newRow)) {
        newRow[columnIndex] = editingValue;
      } else {
        newRow[column] = editingValue;
      }

      // Create new array reference to trigger Svelte reactivity
      displayRows = [
        ...displayRows.slice(0, rowIndex),
        newRow,
        ...displayRows.slice(rowIndex + 1),
      ];

      // IMPORTANT: Also update displayData.rows to keep it in sync
      const newDisplayDataRows = [...displayData.rows];
      newDisplayDataRows[rowIndex] = newRow;
      displayData = { ...displayData, rows: newDisplayDataRows };

      // Track the edit (update editedRows only)
      // Convert value to proper type before storing
      const convertedValue = convertValueToColumnType(editingValue, column);

      if (!editedRows.has(rowIndex)) {
        editedRows.set(rowIndex, new Map());
      }
      editedRows.get(rowIndex).set(column, convertedValue);
      editedRows = new Map(editedRows);

      console.log(
        `[handleCellBlur] Cell edited - Row ${rowIndex}, Column ${column}:`,
        {
          original: editingValue,
          converted: convertedValue,
          type: typeof convertedValue,
        }
      );
    }

    // Always clear editing state
    cancelEdit();
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

    // Check if value has actually changed - handle null/undefined properly
    const originalStr =
      originalValue === null || originalValue === undefined
        ? ""
        : String(originalValue);
    const hasChanged = popupEditorValue !== originalStr;

    if (!hasChanged) {
      // No changes, just close
      closePopupEditor();
      return;
    }

    // VALIDATE: Check if new value is valid for column type
    if (!isValidForColumnType(popupEditorValue, column)) {
      console.warn(
        `[savePopupEdit] ❌ Invalid value for column ${column}: "${popupEditorValue}"`
      );
      closePopupEditor();
      alert(
        `❌ Invalid value for ${column}. Please enter a valid ${displayData.columns?.find((c) => c.name === column)?.data_type || "value"}.`
      );
      return;
    }

    // IMPORTANT: Backup original data BEFORE updating displayData
    // (first time edit on this row)
    if (!originalRowData.has(rowIndex)) {
      const rowToBackup = displayData.rows[rowIndex];
      let colNames = displayData.column_names || [];
      if (colNames.length === 0 && displayData.columns) {
        colNames = displayData.columns.map((col) => col.name);
      }

      // Convert array to object if needed
      const backupData = Array.isArray(rowToBackup)
        ? Object.fromEntries(
            colNames.map((name, idx) => [name, rowToBackup[idx]])
          )
        : { ...rowToBackup };

      originalRowData.set(rowIndex, backupData);
      console.log(
        `[savePopupEdit] Backed up original row ${rowIndex}:`,
        backupData
      );
    }

    // Update displayRows first with the new value
    // column is a string (column name), not an object
    const columnIndex = columnNames.indexOf(column);
    const newRow = Array.isArray(displayRows[rowIndex])
      ? [...displayRows[rowIndex]]
      : { ...displayRows[rowIndex] };

    if (Array.isArray(newRow)) {
      newRow[columnIndex] = popupEditorValue;
    } else {
      newRow[column] = popupEditorValue;
    }

    // Create new array reference to trigger Svelte reactivity
    displayRows = [
      ...displayRows.slice(0, rowIndex),
      newRow,
      ...displayRows.slice(rowIndex + 1),
    ];

    // IMPORTANT: Also update displayData.rows to keep it in sync
    const newDisplayDataRows = [...displayData.rows];
    newDisplayDataRows[rowIndex] = newRow;
    displayData = { ...displayData, rows: newDisplayDataRows };

    // Track the edit (update editedRows only)
    // Convert value to proper type before storing
    const convertedValue = convertValueToColumnType(popupEditorValue, column);

    if (!editedRows.has(rowIndex)) {
      editedRows.set(rowIndex, new Map());
    }
    editedRows.get(rowIndex).set(column, convertedValue);
    editedRows = new Map(editedRows);

    console.log(
      `[savePopupEdit] Cell edited - Row ${rowIndex}, Column ${column}:`,
      {
        original: popupEditorValue,
        converted: convertedValue,
        type: typeof convertedValue,
      }
    );

    closePopupEditor();
  }

  function cancelAllEdits() {
    console.log("[cancelAllEdits] Before cancel:", {
      originalRowDataSize: originalRowData.size,
      originalRowData: Array.from(originalRowData.entries()),
      displayDataRowsCount: displayData.rows.length,
      newRowsCount: newRows.size,
      deletedRowsCount: deletedRows.size,
    });

    const {
      displayData: newDisplayData,
      editedRows: newEditedRows,
      originalRowData: newOriginalRowData,
    } = cancelAllEditsService(displayData, editedRows, originalRowData);

    console.log("[cancelAllEdits] After cancel service:", {
      displayDataRowsCount: newDisplayData.rows.length,
      firstRow: newDisplayData.rows[0],
    });

    // Remove _isDeleted and _isNewRow flags from all rows, AND filter out new rows
    const cleanedDisplayData = {
      ...newDisplayData,
      rows: newDisplayData.rows
        .filter((row) => {
          // Filter out rows that are marked as new
          if (Array.isArray(row)) {
            return true; // Keep array rows
          } else {
            return !row._isNewRow; // Remove rows with _isNewRow flag
          }
        })
        .map((row) => {
          if (Array.isArray(row)) {
            // For array rows, just return as-is
            return row;
          } else {
            // For object rows, remove internal flags
            const cleanRow = { ...row };
            delete cleanRow._isDeleted;
            delete cleanRow._isNewRow;
            delete cleanRow._rowId;
            return cleanRow;
          }
        }),
    };

    displayData = cleanedDisplayData;
    displayRows = cleanedDisplayData.rows;
    editedRows = newEditedRows;
    originalRowData = newOriginalRowData;

    console.log("[cancelAllEdits] After cleanup:", {
      displayDataRowsCount: displayData.rows.length,
      newRowsCleared: newRows.size === 0,
      deletedRowsCleared: deletedRows.size === 0,
    });

    // Clear new rows and deleted rows
    newRows.clear();
    newRows = new Map();
    deletedRows.clear();
    deletedRows = new Set();

    cancelEdit();
  }

  async function handleQueryEdit(newQuery) {
    try {
      // Update finalQuery with edited query
      finalQuery = newQuery;

      // If not in table mode, reload data with the new query
      if (!isTableMode && newQuery) {
        isLoadingData = true;
        displayData = null;
        currentOffset = 0;
        hasMoreData = true;

        // Reload with the edited query
        await loadQueryInitial({
          connection,
          query: newQuery,
          limit: paginateLimit,
          offset: 0,
        });
      }
    } catch (error) {
      console.error("Failed to execute edited query:", error);
      alert(`Failed to execute query: ${error}`);
    } finally {
      isLoadingData = false;
    }
  }

  function toggleViewMode() {
    viewMode = viewMode === "grid" ? "json" : "grid";
    if (tabId) {
      tabDataStore.setViewMode(tabId, viewMode);
    }
  }
</script>

<div class="data-grid-container h-100 d-flex flex-column">
  {#if isLoadingData}
    <div
      class="d-flex flex-column align-items-center justify-content-center h-100 text-primary"
    >
      <i class="fas fa-spinner fa-spin fa-3x mb-3"></i>
      <p class="fs-5">Loading table data...</p>
    </div>
  {:else if displayData && displayData.columns && displayData.columns.length > 0}
    <!-- Show table (with or without rows) -->
    <DataGridHeader
      {finalQuery}
      {executedQuery}
      {viewMode}
      {columnFilters}
      databaseType={connection?.db_type}
      onViewModeToggle={toggleViewMode}
      onClearFilters={handleClearAllFilters}
      onQueryEdit={handleQueryEdit}
    />

    {#if viewMode === "grid"}
      <GridView
        bind:tableWrapper
        bind:headerWrapper
        bind:rowNumbersWrapper
        {displayData}
        {displayNames}
        {displayRows}
        {columnNames}
        {isLoadingMore}
        {hasMoreData}
        {editedRows}
        bind:editingCell
        bind:editingValue
        {originalRowData}
        {selectedCell}
        {selectedRows}
        {sortStack}
        {columnFilters}
        {selectedFilterValues}
        onLoadMore={handleLoadMore}
        onSort={handleSortClick}
        onFilter={handleFilterClick}
        onFilterInput={handleFilterInput}
        onCellClick={handleCellClick}
        onCellDoubleClick={handleCellDoubleClick}
        onCellBlur={handleCellBlur}
        onCellKeydown={handleCellKeydown}
        onScroll={handleGridScroll}
        onRowNumberClick={handleRowNumberClick}
      />
    {:else}
      <JsonView
        {displayRows}
        {displayData}
        {isLoadingMore}
        {hasMoreData}
        onLoadMore={handleLoadMore}
        onScroll={handleGridScroll}
      />
    {/if}

    <DataGridFooter
      displayRowsLength={displayRows.length}
      {displayData}
      onCancelChanges={cancelAllEdits}
      connectionId={connection?.id}
      database={databaseName}
      table={tableName}
      schema={schemaName}
      {newRows}
      {editedRows}
      {deletedRows}
      {displayRows}
      {selectedRows}
      {selectedCell}
      {originalRowData}
      columns={displayData.columns}
      {paginateLimit}
      {executionTime}
      {fetchTime}
      onPaginateLimitChange={(newLimit) => {
        paginateLimit = newLimit;
        // Reset pagination state when limit changes
        currentOffset = 0;
        hasMoreData = true;
        // Reload data with new limit
        handleReloadData();
      }}
      onDisplayDataChange={(newData) => {
        // Update displayData.rows with new data
        displayData = { ...displayData, rows: newData };
        // Force update displayRows to trigger reactivity
        displayRows = [...newData];
      }}
      onRefreshData={async () => {
        if (isTableMode) {
          await handleReloadData();
        } else {
          await handleReloadData();
        }
      }}
      onSetupAutoRefresh={setupAutoRefresh}
      onStopAutoRefresh={stopAutoRefresh}
      {isAutoRefreshActive}
      onEditCell={handleEditCellButton}
    />
  {:else if displayData && displayData.rows_affected !== null && displayData.rows_affected !== undefined}
    <!-- Show INSERT/UPDATE/DELETE result -->
    <div
      class="d-flex flex-column align-items-center justify-content-center h-100 text-secondary"
    >
      <i class="fas fa-info-circle fa-3x mb-3 opacity-50"></i>
      <p class="fs-5">Query executed successfully</p>
      <span class="badge bg-success"
        >{displayData.rows_affected} rows affected</span
      >
    </div>
  {:else}
    <!-- No data to display -->
    <div
      class="d-flex flex-column align-items-center justify-content-center h-100 text-secondary"
    >
      <i class="fas fa-table fa-3x mb-3 opacity-25"></i>
      <p class="fs-5">No data to display</p>
      <p class="text-muted">Execute a query to see results here</p>
    </div>
  {/if}
</div>

<FilterModal
  show={showFilterModal}
  column={filterModalColumn}
  position={filterModalPosition}
  selectedValues={selectedFilterValues[filterModalColumn] || new Set()}
  availableValues={filterValuesCache[
    `${filterModalColumn}_${filterSearchQuery || ""}`
  ] || getDistinctValues(displayData, filterModalColumn)}
  loading={loadingFilterValues}
  searchQuery={filterSearchQuery}
  on:close={closeFilterModal}
  on:apply={handleApplyFilter}
  on:clear={handleClearFilter}
  on:search={(e) => {
    filterSearchQuery = e.detail.query;
    handleLoadFilterValues(filterModalColumn, e.detail.query);
  }}
  on:selectionChange={(e) => {
    selectedFilterValues[e.detail.column] = e.detail.selectedValues;
    selectedFilterValues = { ...selectedFilterValues };
  }}
/>

<CellEditorModal
  show={showPopupEditor}
  value={popupEditorValue}
  column={popupEditingCell?.column}
  rowIndex={popupEditingCell?.rowIndex}
  on:close={closePopupEditor}
  on:save={(e) => {
    popupEditorValue = e.detail.value;
    savePopupEdit();
  }}
/>

<style>
  .data-grid-container {
    position: relative;
    overflow: hidden;
  }
</style>
