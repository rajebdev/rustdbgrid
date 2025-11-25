<script>
  import { onMount, onDestroy } from "svelte";
  import { tabDataStore } from "../stores/tabData";
  import { activeConnection } from "../stores/connections";
  import { getFilterValues, executeQueryWithFilters } from "../utils/tauri";

  export let data = null;
  export let tabId = null;
  export let executedQuery = "";

  // Debug props
  $: {
    console.log("📊 DataGrid Props Updated:");
    console.log("  - tabId:", tabId);
    console.log(
      "  - executedQuery:",
      executedQuery ? `"${executedQuery.substring(0, 50)}..."` : "EMPTY/NULL"
    );
    console.log("  - data rows:", data?.rows?.length || 0);
    console.log("  - activeConnection:", $activeConnection ? "EXISTS" : "NULL");
  }

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

  // Load saved state when tab changes
  $: if (tabId && $tabDataStore[tabId]) {
    const savedState = $tabDataStore[tabId];
    columnFilters = savedState.filters || {};
    sortColumn = savedState.sortColumn || null;
    sortDirection = savedState.sortDirection || "asc";

    // Restore scroll position after a short delay
    if (tableWrapper && savedState.scrollPosition) {
      setTimeout(() => {
        tableWrapper.scrollTop = savedState.scrollPosition || 0;
      }, 50);
    }
  }

  // Clear cache when executedQuery changes
  $: if (executedQuery) {
    filterValuesCache = {};
    columnFilters = {};
    sortColumn = null;
    sortDirection = "asc";
  }

  async function reloadDataWithFilters() {
    // Only reload if we have filters or sorting applied
    const hasFilters = Object.keys(columnFilters).length > 0;
    const hasSorting = sortColumn !== null;

    if (!hasFilters && !hasSorting) {
      return; // Use original data
    }

    // Validate executedQuery is not empty
    if (!executedQuery || executedQuery.trim() === "") {
      console.log("⚠️ Cannot reload - executedQuery is empty");
      return;
    }

    if (!$activeConnection) {
      console.log("⚠️ Cannot reload - no active connection");
      return;
    }

    isLoadingData = true;
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

      const result = await executeQueryWithFilters(
        $activeConnection,
        executedQuery,
        Object.keys(filters).length > 0 ? filters : null,
        sortColumn,
        sortColumn ? sortDirection.toUpperCase() : null,
        200 // Default limit 200 rows
      );

      // Update data with filtered results
      data = result;

      // Store the final query from backend
      if (result.final_query) {
        finalQuery = result.final_query;
      }
    } catch (error) {
      console.error("❌ Failed to reload data with filters:", error);
    } finally {
      isLoadingData = false;
    }
  }

  // Save scroll position when scrolling
  function handleScroll() {
    if (tabId && tableWrapper) {
      tabDataStore.setScrollPosition(tabId, tableWrapper.scrollTop);
    }
  }

  function formatValue(value) {
    if (value === null || value === undefined) {
      return "NULL";
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
      if (typeof value === "number") {
        numericCount++;
      } else if (typeof value === "string") {
        // Check if string is a valid number
        const trimmed = value.trim();
        if (trimmed !== "" && !isNaN(Number(trimmed))) {
          numericCount++;
        }
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
    console.log("🔌 activeConnection:", $activeConnection);

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
    if (!executedQuery || executedQuery.trim() === "" || !$activeConnection) {
      console.log("⚠️ Skipping server-side filter:");
      console.log(
        "  - executedQuery:",
        executedQuery ? `"${executedQuery.substring(0, 50)}..."` : "EMPTY"
      );
      console.log(
        "  - activeConnection:",
        $activeConnection ? "EXISTS" : "NULL"
      );
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
        $activeConnection,
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

    if (tabId) {
      tabDataStore.setFilters(tabId, {});
      tabDataStore.setSortState(tabId, null, "asc");
    }

    // Trigger server-side reload (will load original data without filters)
    reloadDataWithFilters();
  }

  // Watch for search query changes and load from server
  let searchDebounceTimeout;
  $: if (showFilterModal && filterModalColumn) {
    clearTimeout(searchDebounceTimeout);
    searchDebounceTimeout = setTimeout(() => {
      if (executedQuery && $activeConnection) {
        loadFilterValuesFromServer(
          filterModalColumn,
          filterSearchQuery || null
        );
      }
    }, 300);
  }

  // Display rows directly - filtering and sorting should be done on server-side
  $: displayRows = data?.rows || [];
</script>

<div class="data-grid-container h-100 d-flex flex-column">
  {#if isLoadingData}
    <div
      class="d-flex flex-column align-items-center justify-content-center h-100 text-primary"
    >
      <i class="fas fa-spinner fa-spin fa-3x mb-3"></i>
      <p class="fs-5">Loading filtered data...</p>
    </div>
  {:else if data && data.rows.length > 0}
    <div class="d-flex align-items-center gap-2 p-2 bg-light border-bottom">
      <span class="badge bg-primary">
        <i class="fas fa-table"></i> Rows: {displayRows.length}
      </span>
      <span class="badge bg-info">
        <i class="fas fa-columns"></i> Columns: {data.columns.length}
      </span>
      <span class="badge bg-secondary">
        <i class="fas fa-clock"></i>
        {data.execution_time}ms
      </span>
      {#if Object.keys(columnFilters).length > 0}
        <button
          class="btn btn-sm btn-danger ms-auto"
          on:click={clearAllFilters}
        >
          <i class="fas fa-times"></i> Clear filters
        </button>
      {/if}
    </div>

    <div
      class="table-wrapper flex-grow-1 overflow-auto"
      bind:this={tableWrapper}
      on:scroll={handleScroll}
    >
      <table
        class="table table-sm table-hover table-bordered data-table mb-0"
        style="table-layout: auto;"
      >
        <thead class="table-light sticky-top">
          <tr>
            <th class="text-center" style="width: 50px;">#</th>
            {#each data.columns as column}
              {@const isNumeric = isNumericColumn(column)}
              <th class:text-end={isNumeric}>
                <div
                  class="column-header"
                  class:numeric-header-content={isNumeric}
                >
                  <button
                    class="sort-button"
                    class:numeric-sort={isNumeric}
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
                    class="form-control form-control-sm {isNumeric
                      ? 'text-end'
                      : ''}"
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
            <tr>
              <td class="text-center text-muted fw-medium">{index + 1}</td>
              {#each data.columns as column}
                <td
                  class="{row[column] === null || row[column] === undefined
                    ? 'null-value fst-italic'
                    : ''} {isNumericColumn(column)
                    ? 'text-end font-monospace'
                    : ''}"
                  title={formatValue(row[column])}
                >
                  {formatValue(row[column])}
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else if data}
    <div
      class="d-flex flex-column align-items-center justify-content-center h-100 text-secondary"
    >
      <i class="fas fa-info-circle fa-3x mb-3 opacity-50"></i>
      <p class="fs-5">Query executed successfully</p>
      {#if data.rows_affected !== null}
        <span class="badge bg-success">{data.rows_affected} rows affected</span>
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

<style>
  /* Custom scrollbar styling */
  .table-wrapper::-webkit-scrollbar {
    width: 12px;
    height: 12px;
  }

  .table-wrapper::-webkit-scrollbar-track {
    background: #f8f9fa;
  }

  .table-wrapper::-webkit-scrollbar-thumb {
    background: #c0c0c0;
    border-radius: 6px;
  }

  .table-wrapper::-webkit-scrollbar-thumb:hover {
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

  .numeric-header-content {
    flex-direction: row-reverse;
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

  .numeric-sort {
    flex-direction: row-reverse;
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

  /* Table wrapper */
  .table-wrapper {
    position: relative;
  }

  /* Ensure table cells truncate properly */
  .data-table td,
  .data-table th {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
