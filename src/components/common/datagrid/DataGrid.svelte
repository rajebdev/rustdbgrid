<script>
  import { onMount, afterUpdate } from "svelte";
  import { tabDataStore } from "../../../stores/tabData";
  import { loadTableData } from "../../../utils/tauri";
  import { DatabaseType } from "../../../utils/databaseTypes";

  // Views
  import GridView from "./views/GridView.svelte";
  import JsonView from "./views/JsonView.svelte";

  // Partials
  import DataGridHeader from "./partials/DataGridHeader.svelte";
  import DataGridFooter from "./partials/DataGridFooter.svelte";
  import EditFooter from "./partials/EditFooter.svelte";

  // Modals
  import SqlPreviewModal from "../../modals/SqlPreviewModal.svelte";
  import FilterModal from "../../modals/FilterModal.svelte";
  import CellEditorModal from "../../modals/CellEditorModal.svelte";

  // Services
  import {
    loadTableDataWithFilters,
    reloadDataWithFilters,
    loadMoreData,
    loadFilterValuesFromServer,
  } from "../../../services/dataGridService";
  import {
    startEdit,
    trackEditedRow,
    cancelAllEdits as cancelAllEditsService,
    generateUpdateSql,
  } from "../../../services/gridEditService";
  import {
    openFilterModal as openFilterModalService,
    getDistinctValues,
  } from "../../../services/gridFilterService";
  import { handleSort } from "../../../services/gridSortService";

  // Utils
  import {
    syncColumnWidths,
    isScrolledNearBottom,
    isScrollingDown,
    shouldLoadMore,
  } from "../../../utils/gridScrollSync";

  export let data = null;
  export let tabId = null;
  export let executedQuery = "";
  export let connection = null;
  export let tableName = "";
  export let databaseName = "";
  export let schemaName = "";

  // State
  let displayData = null;
  let isFiltered = false;
  let totalRows = 0;
  let currentOffset = 0;
  let isLoadingMore = false;
  let hasMoreData = true;
  let lastLoadedQueryId = null;
  let columnFilters = {};
  let sortColumn = null;
  let sortDirection = "asc";
  let isCustomQuery = false; // Flag to distinguish custom SQL vs table query
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

  // Editing state
  let editingCell = null;
  let editingValue = "";
  let originalValue = "";
  let editedRows = new Map();
  let originalRowData = new Map();
  let showSqlPreview = false;
  let previewSql = "";
  let pendingUpdates = [];
  let showPopupEditor = false;
  let popupEditorValue = "";
  let popupEditingCell = null;

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

  $: columnTypes =
    displayData?.columns?.reduce((acc, col) => {
      if (typeof col === "object" && col.name && col.data_type) {
        acc[col.name] = col.data_type;
      }
      return acc;
    }, {}) || {};

  $: columnNames =
    displayData?.columns?.map((col) =>
      typeof col === "string" ? col : col.name
    ) || [];

  $: displayRows = displayData?.rows || [];
  $: hasUnsavedEdits = editedRows.size > 0;

  // Set default view mode
  $: if (connection && !$tabDataStore[tabId]?.viewMode) {
    viewMode = connection.db_type === DatabaseType.MONGODB ? "json" : "grid";
  }

  // Load saved state when tab changes
  $: if (tabId && tabId !== currentTabId) {
    currentTabId = tabId;

    if ($tabDataStore[tabId]) {
      const savedState = $tabDataStore[tabId];
      columnFilters = savedState.filters || {};
      sortColumn = savedState.sortColumn || null;
      sortDirection = savedState.sortDirection || "asc";
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

  // Update display data
  $: if (data && !isFiltered && !isLoadingMore) {
    const queryId =
      executedQuery +
      JSON.stringify(columnFilters) +
      sortColumn +
      sortDirection;
    if (queryId !== lastLoadedQueryId) {
      displayData = data;
      totalRows = data?.total_count || data?.rows?.length || 0;
      currentOffset = data?.rows?.length || 0;
      hasMoreData = data?.rows?.length === 200;
      lastLoadedQueryId = queryId;
      // Determine if this is a custom query (from SQL editor) or table query (from sidebar)
      isCustomQuery =
        !tableName && executedQuery && executedQuery.trim() !== "";
    }
  }

  // Clear cache when query changes
  $: if (executedQuery) {
    filterValuesCache = {};
    columnFilters = {};
    sortColumn = null;
    sortDirection = "asc";
    currentOffset = 0;
    hasMoreData = true;
    lastLoadedQueryId = null;
    isCustomQuery = !tableName && executedQuery.trim() !== "";
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

    return () => {
      if (resizeObserver) {
        resizeObserver.disconnect();
      }
    };
  });

  $: if (tableWrapper && resizeObserver) {
    resizeObserver.observe(tableWrapper);
  }

  async function handleReloadData() {
    const previousData = displayData;
    displayData = null;
    isLoadingData = true;
    isFiltered = Object.keys(columnFilters).length > 0 || sortColumn !== null;
    currentOffset = 0;
    hasMoreData = true;

    await new Promise((resolve) => setTimeout(resolve, 0));

    try {
      let result;

      // If this is a table query (from sidebar), use loadTableDataWithFilters
      if (tableName && !isCustomQuery) {
        result = await loadTableDataWithFilters(
          connection,
          tableName,
          databaseName,
          schemaName,
          columnFilters,
          sortColumn,
          sortDirection
        );
        lastLoadedQueryId =
          tableName +
          JSON.stringify(columnFilters) +
          sortColumn +
          sortDirection;
      }
      // If this is a custom query (from SQL editor), use reloadDataWithFilters
      else if (executedQuery && executedQuery.trim() !== "") {
        result = await reloadDataWithFilters(
          connection,
          executedQuery,
          tableName,
          databaseName,
          columnFilters,
          sortColumn,
          sortDirection
        );
        lastLoadedQueryId =
          executedQuery +
          JSON.stringify(columnFilters) +
          sortColumn +
          sortDirection;
      } else {
        // No valid query or table
        displayData = previousData;
        isLoadingData = false;
        return;
      }

      displayData = result;
      totalRows = result?.total_count || result?.rows?.length || 0;
      currentOffset = result?.rows?.length || 0;
      hasMoreData = result?.rows?.length === 200;

      if (result.final_query) {
        finalQuery = result.final_query;
      }
    } catch (error) {
      console.error("❌ Failed to reload data:", error);
      displayData = previousData;
      isFiltered = false;
    } finally {
      isLoadingData = false;
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

    try {
      const result = await loadMoreData(
        connection,
        tableName,
        databaseName,
        schemaName,
        columnFilters,
        sortColumn,
        sortDirection,
        currentOffset,
        executedQuery // Pass executedQuery for custom queries
      );

      if (result?.rows && displayData) {
        displayData.rows = [...displayData.rows, ...result.rows];
        currentOffset += result.rows.length;
        hasMoreData = result.has_more_data;
      }
    } catch (error) {
      console.error("❌ Failed to load more data:", error);
      hasMoreData = false;
    } finally {
      isLoadingMore = false;
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
  function handleSortClick(column) {
    const result = handleSort(
      column,
      sortColumn,
      sortDirection,
      tabId,
      tabDataStore,
      handleReloadData
    );
    sortColumn = result.sortColumn;
    sortDirection = result.sortDirection;
  }

  // Filtering
  function handleFilterClick(column, event) {
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

    closeFilterModal();
    handleReloadData();
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
    closeFilterModal();
    handleReloadData();
  }

  function closeFilterModal() {
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

  function handleFinishEdit() {
    if (!editingCell) return;

    const { rowIndex, column } = editingCell;

    if (
      editingValue === originalValue ||
      (editingValue === "" &&
        (originalValue === null || originalValue === undefined))
    ) {
      cancelEdit();
      return;
    }

    const { displayData: newDisplayData, editedRows: newEditedRows } =
      trackEditedRow(
        rowIndex,
        column,
        editingValue,
        displayData,
        editedRows,
        originalRowData
      );

    displayData = newDisplayData;
    editedRows = newEditedRows;

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

    if (
      popupEditorValue === originalValue ||
      (popupEditorValue === "" &&
        (originalValue === null || originalValue === undefined))
    ) {
      closePopupEditor();
      return;
    }

    const { displayData: newDisplayData, editedRows: newEditedRows } =
      trackEditedRow(
        rowIndex,
        column,
        popupEditorValue,
        displayData,
        editedRows,
        originalRowData
      );

    displayData = newDisplayData;
    editedRows = newEditedRows;

    closePopupEditor();
  }

  function cancelAllEdits() {
    const {
      displayData: newDisplayData,
      editedRows: newEditedRows,
      originalRowData: newOriginalRowData,
    } = cancelAllEditsService(displayData, editedRows, originalRowData);

    displayData = newDisplayData;
    editedRows = newEditedRows;
    originalRowData = newOriginalRowData;

    cancelEdit();
  }

  function handleShowPreview() {
    pendingUpdates = generateUpdateSql(displayData, editedRows, executedQuery);
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

  async function handleExecuteUpdates() {
    if (pendingUpdates.length === 0 || !connection) return;

    try {
      for (const update of pendingUpdates) {
        // Execute UPDATE/DELETE/INSERT statements using loadTableData with subquery
        await loadTableData(
          connection.id,
          connection.db_type,
          `RustDBGridQuery(${update.sql})`,
          { limit: 1, offset: 0 }
        );
      }

      editedRows.clear();
      editedRows = new Map();

      await handleReloadData();

      closeSqlPreview();

      alert("Updates executed successfully!");
    } catch (error) {
      console.error("Failed to execute updates:", error);
      alert(`Failed to execute updates: ${error}`);
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
      <p class="fs-5">Loading filtered data...</p>
    </div>
  {:else if displayData && displayData.rows.length > 0}
    <DataGridHeader
      {finalQuery}
      {executedQuery}
      {viewMode}
      {columnFilters}
      {hasUnsavedEdits}
      editedRowsSize={editedRows.size}
      onViewModeToggle={toggleViewMode}
      onClearFilters={handleClearAllFilters}
      onSaveChanges={handleShowPreview}
      onCancelChanges={cancelAllEdits}
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
        {editingCell}
        {editingValue}
        {originalRowData}
        {sortColumn}
        {sortDirection}
        {columnFilters}
        {selectedFilterValues}
        onLoadMore={handleLoadMore}
        onSort={handleSortClick}
        onFilter={handleFilterClick}
        onFilterInput={handleFilterInput}
        onCellDoubleClick={handleCellDoubleClick}
        onScroll={handleGridScroll}
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

    <EditFooter
      {hasUnsavedEdits}
      editedRowsSize={editedRows.size}
      onSaveChanges={handleShowPreview}
      onCancelChanges={cancelAllEdits}
    />

    <DataGridFooter
      {totalRows}
      displayRowsLength={displayRows.length}
      columnCount={displayData.columns.length}
      executionTime={displayData.execution_time || "0"}
      {displayData}
    />
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
</div>

<!-- Modals -->
<SqlPreviewModal
  show={showSqlPreview}
  {pendingUpdates}
  {previewSql}
  on:close={closeSqlPreview}
  on:execute={handleExecuteUpdates}
/>

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
