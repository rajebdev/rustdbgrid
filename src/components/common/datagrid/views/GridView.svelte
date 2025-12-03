<script>
  import { afterUpdate } from "svelte";
  import {
    syncColumnWidths,
    isScrolledNearBottom,
    isScrollingDown,
    shouldLoadMore,
  } from "../../../../utils/gridScrollSync";
  import { formatValue } from "../../../../utils/dataFormatters";
  import ArrayCell from "../../ArrayCell.svelte";

  export let displayData = null;
  export let displayNames = [];
  export let displayRows = [];
  export let columnNames = [];
  export let isLoadingMore = false;
  export let hasMoreData = true;
  export let editedRows = new Map();
  export let editingCell = null;
  export let editingValue = "";
  export let originalRowData = new Map();
  export let sortColumn = null;
  export let sortDirection = "asc";
  export let columnFilters = {};
  export let selectedFilterValues = {};

  export let onLoadMore = null;
  export let onSort = null;
  export let onFilter = null;
  export let onFilterInput = null;
  export let onCellClick = null;
  export let onCellDoubleClick = null;
  export let onScroll = null;

  let tableWrapper;
  let headerWrapper;
  let rowNumbersWrapper;
  let lastScrollTop = 0;
  let lastLoadTriggeredAt = 0;
  let isRestoringScroll = false;
  let filterInputValues = {}; // Local state for input values before Enter

  // Track if filters are active (used for accessibility)
  $: hasActiveFilters = Object.keys(selectedFilterValues).some(
    (col) => selectedFilterValues[col] && selectedFilterValues[col].size > 0
  );

  // Detect if column is numeric by sampling first 10 rows
  function isNumericColumn(column) {
    if (!displayData?.rows || displayData.rows.length === 0) return false;

    const sampleSize = Math.min(10, displayData.rows.length);
    let numericCount = 0;

    for (let i = 0; i < sampleSize; i++) {
      const value = displayData.rows[i][column];
      if (value === null || value === undefined) continue;
      if (typeof value === "number") {
        numericCount++;
      }
    }

    return numericCount / sampleSize > 0.7;
  }

  afterUpdate(() => {
    if (displayData?.rows?.length > 0) {
      syncColumnWidths(headerWrapper, tableWrapper);
    }
  });

  function handleScroll() {
    if (!tableWrapper || isRestoringScroll) return;

    const currentScrollTop = tableWrapper.scrollTop;
    const currentScrollLeft = tableWrapper.scrollLeft;

    // Sync horizontal scroll with header
    if (headerWrapper) {
      headerWrapper.scrollLeft = currentScrollLeft;
    }

    // Sync vertical scroll with row numbers column
    if (rowNumbersWrapper) {
      rowNumbersWrapper.scrollTop = currentScrollTop;
    }

    // Check if should load more
    const scrolledToBottom = isScrolledNearBottom(tableWrapper, 200);
    const scrolling = isScrollingDown(currentScrollTop, lastScrollTop);
    const now = Date.now();
    const timeSinceLastLoad = now - lastLoadTriggeredAt;

    if (
      shouldLoadMore(
        scrolledToBottom,
        hasMoreData,
        isLoadingMore,
        scrolling,
        timeSinceLastLoad,
        1000
      )
    ) {
      lastLoadTriggeredAt = now;
      if (onLoadMore) onLoadMore();
    }

    if (Math.abs(currentScrollTop - lastScrollTop) >= 5) {
      lastScrollTop = currentScrollTop;
      if (onScroll)
        onScroll({
          scrollTop: currentScrollTop,
          scrollLeft: currentScrollLeft,
        });
    }
  }

  function handleSortClick(column) {
    if (onSort) {
      onSort(column);
    }
  }

  function handleFilterClick(column, event) {
    if (onFilter) {
      onFilter(column, event);
    }
  }

  function handleCellClick(rowIndex, column, event) {
    if (onCellClick) {
      onCellClick(rowIndex, column, event);
    }
  }

  function handleCellDoubleClick(rowIndex, column, currentValue, event) {
    if (onCellDoubleClick) {
      onCellDoubleClick(rowIndex, column, currentValue, event);
    }
  }

  function handleFilterKeydown(column, event) {
    if (event.key === "Enter") {
      event.preventDefault();
      if (onFilterInput) {
        onFilterInput(column, event.target.value);
      }
    }
  }

  function handleFilterInputChange(column, event) {
    filterInputValues[column] = event.target.value;
    filterInputValues = { ...filterInputValues };
  }

  function getFilterDisplayValue(column) {
    if (Array.isArray(columnFilters[column])) {
      return `${columnFilters[column].length} selected`;
    }
    // Use local input value if exists, otherwise use filter value
    if (filterInputValues[column] !== undefined) {
      return filterInputValues[column];
    }
    return columnFilters[column] || "";
  }

  // Sync local input values with actual filters
  $: {
    if (columnFilters) {
      Object.keys(columnFilters).forEach((col) => {
        if (
          !Array.isArray(columnFilters[col]) &&
          filterInputValues[col] === undefined
        ) {
          filterInputValues[col] = columnFilters[col];
        }
      });
    }
  }
</script>

<div
  class="table-container flex-grow-1"
  class:filters-active={hasActiveFilters}
  data-filters={hasActiveFilters ? "active" : "none"}
>
  <!-- Row numbers column -->
  <div class="row-numbers-column">
    <div class="row-number-header-cell">#</div>
    <div class="row-numbers-body" bind:this={rowNumbersWrapper}>
      {#each displayRows as row, index}
        <div
          class="row-number-cell"
          class:row-even={index % 2 === 0}
          class:row-odd={index % 2 !== 0}
          class:edited={originalRowData.has(index)}
        >
          {index + 1}
        </div>
      {/each}
      <div
        class="row-number-cell"
        class:row-even={true}
        style="height: 100px;"
      ></div>
    </div>
  </div>

  <!-- Main data area -->
  <div class="data-area">
    <!-- Fixed Header Table -->
    <div class="table-header-wrapper" bind:this={headerWrapper}>
      <table
        class="table table-sm table-bordered data-table header-table mb-0"
        style="table-layout: auto;"
      >
        <thead>
          <tr>
            {#each columnNames as column, idx}
              {@const displayName = displayNames[idx] || column}
              {@const isNumeric = isNumericColumn(column)}
              <th class:text-end={isNumeric}>
                <div class="column-header">
                  <button
                    class="sort-button"
                    class:numeric-sort={isNumeric}
                    on:click={() => handleSortClick(column)}
                  >
                    <span class="column-name">{displayName}</span>
                    {#if sortColumn === column}
                      <i
                        class="fas {sortDirection === 'asc'
                          ? 'fa-sort-up'
                          : 'fa-sort-down'} sort-icon"
                      ></i>
                    {:else}
                      <i class="fas fa-sort sort-icon inactive"></i>
                    {/if}
                  </button>
                  {#if Object.keys(columnFilters).includes(column)}
                    <button
                      class="filter-icon-button active"
                      on:click={(e) => handleFilterClick(column, e)}
                      title="Filter active"
                    >
                      <i class="fas fa-filter"></i>
                    </button>
                  {:else}
                    <button
                      class="filter-icon-button"
                      on:click={(e) => handleFilterClick(column, e)}
                      title="Add filter"
                    >
                      <i class="fas fa-filter"></i>
                    </button>
                  {/if}
                </div>
                <div class="filter-input-wrapper">
                  <input
                    type="text"
                    class="filter-input {isNumeric ? 'text-end' : ''}"
                    placeholder="Press Enter to filter..."
                    value={getFilterDisplayValue(column)}
                    readonly={Array.isArray(columnFilters[column])}
                    on:input={(e) => handleFilterInputChange(column, e)}
                    on:keydown={(e) => handleFilterKeydown(column, e)}
                  />
                </div>
              </th>
            {/each}
            <th></th>
          </tr>
        </thead>
      </table>
    </div>

    <!-- Scrollable Body Table -->
    <div
      class="table-body-wrapper"
      bind:this={tableWrapper}
      on:scroll={handleScroll}
      on:wheel={handleScroll}
      role="grid"
      aria-label="Data grid"
    >
      <table
        class="table table-sm table-bordered data-table body-table mb-0"
        style="table-layout: auto;"
      >
        <tbody>
          {#each displayRows as row, rowIndex}
            <tr
              class:edited-row={originalRowData.has(rowIndex)}
              class:row-even={rowIndex % 2 === 0}
              class:row-odd={rowIndex % 2 !== 0}
            >
              {#each columnNames as column, colIndex}
                {@const cellValue = Array.isArray(row)
                  ? row[colIndex]
                  : row[column]}
                {@const isEditing =
                  editingCell?.rowIndex === rowIndex &&
                  editingCell?.column === column}
                {@const isEdited = editedRows.get(rowIndex)?.has(column)}
                {@const isNumeric = isNumericColumn(column)}
                {@const isArrayValue = Array.isArray(cellValue)}
                <td
                  class:editing={isEditing}
                  class:edited-cell={isEdited}
                  class:numeric-cell={isNumeric && !isArrayValue}
                  on:click={(e) => handleCellClick(rowIndex, column, e)}
                  on:dblclick={(e) =>
                    handleCellDoubleClick(rowIndex, column, cellValue, e)}
                >
                  {#if isEditing}
                    <input
                      type="text"
                      bind:value={editingValue}
                      on:blur={() => {}}
                    />
                  {:else if cellValue === null || cellValue === undefined}
                    <span class="null-value">{formatValue(cellValue)}</span>
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
        <div class="text-center py-3 bg-light loading-indicator">
          <i class="fas fa-spinner fa-spin text-primary"></i>
          <span class="ms-2 text-muted">Loading more data...</span>
        </div>
      {/if}

      {#if !hasMoreData && displayRows.length > 0}
        <div
          class="text-center py-3 text-muted small bg-light loading-indicator"
        >
          <i class="fas fa-check-circle"></i>
          <span class="ms-2"
            >All data loaded ({displayRows.length.toLocaleString()} rows)</span
          >
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  /* Table container */
  .table-container {
    --scrollbar-size: 8px;
    position: relative;
    overflow: hidden;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .table-header-wrapper {
    flex-shrink: 0;
    overflow-x: auto;
    overflow-y: hidden;
    background-color: var(--grid-header-bg);
    box-shadow: 0 2px 2px -1px rgba(0, 0, 0, 0.1);
    z-index: 10;
    scrollbar-width: none;
    -ms-overflow-style: none;
  }

  .table-header-wrapper::-webkit-scrollbar {
    display: none;
  }

  .table-body-wrapper {
    flex: 1;
    overflow: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--scrollbar-track);
  }

  .table-body-wrapper::-webkit-scrollbar {
    width: var(--scrollbar-size);
    height: var(--scrollbar-size);
  }

  .table-body-wrapper::-webkit-scrollbar-track {
    background: var(--scrollbar-track);
  }

  .table-body-wrapper::-webkit-scrollbar-thumb {
    background: var(--scrollbar-thumb);
    border-radius: 4px;
  }

  .table-body-wrapper::-webkit-scrollbar-thumb:hover {
    background: var(--scrollbar-thumb-hover);
  }

  .data-table {
    table-layout: auto;
    margin: 0;
    width: auto;
    min-width: 100%;
    border-collapse: separate;
    border-spacing: 0;
  }

  .data-table thead th {
    background-color: var(--grid-header-bg);
    color: var(--text-primary);
  }

  .data-table td,
  .data-table th {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 2px 0.5rem;
    border-bottom: 1px solid var(--grid-border);
    max-width: 500px;
    min-width: 100px;
  }

  .row-numbers-column {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 36px;
    display: flex;
    flex-direction: column;
    z-index: 20;
    background-color: var(--grid-header-bg);
    border-right: 1px solid var(--grid-border);
    box-shadow: 2px 0 4px -2px rgba(0, 0, 0, 0.15);
  }

  .row-number-header-cell {
    height: auto;
    min-height: 64px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
    font-size: 11px;
    color: var(--text-secondary);
    background-color: var(--grid-header-bg);
    border-bottom: 1px solid var(--grid-border);
    flex-shrink: 0;
  }

  .row-numbers-body {
    flex: 1;
    overflow: hidden;
    scrollbar-width: none;
    -ms-overflow-style: none;
  }

  .row-numbers-body::-webkit-scrollbar {
    display: none;
  }

  .row-number-cell {
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    color: var(--text-muted);
    border-bottom: 1px solid var(--grid-border);
  }

  .row-number-cell.row-even {
    background-color: var(--grid-row-even);
  }

  .row-number-cell.row-odd {
    background-color: var(--grid-row-odd);
  }

  .row-number-cell.edited {
    background-color: var(--accent-red-light);
    color: var(--accent-red);
    font-weight: 600;
  }

  .data-area {
    position: absolute;
    left: 36px;
    top: 0;
    right: 0;
    bottom: 0;
    display: flex;
    flex-direction: column;
  }

  .data-table tbody tr {
    height: 24px;
  }

  .data-table tbody tr.row-even td {
    background-color: var(--grid-row-even);
  }

  .data-table tbody tr.row-odd td {
    background-color: var(--grid-row-odd);
  }

  .data-table tbody tr.edited-row td {
    background-color: var(--accent-red-light);
    color: var(--accent-red);
  }

  .data-table tbody td {
    height: 24px;
    line-height: 1.5;
    font-size: 12px;
    color: var(--text-primary);
    cursor: pointer;
  }

  .data-table tbody td:hover:not(.edited-cell) {
    background-color: var(--grid-row-hover);
  }

  .data-table tbody td.editing {
    padding: 2px;
    background-color: var(--accent-yellow-light);
  }

  .data-table tbody td.editing input {
    width: 100%;
    height: 100%;
    border: 2px solid var(--accent-yellow);
    padding: 0.25rem;
    background: var(--bg-input);
    color: var(--text-primary);
  }

  .data-table tbody td.edited-cell {
    background-color: var(--accent-red-light);
    color: var(--accent-red);
    border: 2px solid var(--accent-red);
    font-weight: 500;
  }

  /* Numeric column styling - right-align and monospace font */
  .data-table tbody td.numeric-cell {
    text-align: right;
    font-family: var(--bs-font-monospace);
    font-size: 11px;
  }

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
    color: var(--accent-blue);
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
    color: var(--text-muted);
    font-size: 0.875rem;
  }

  .filter-icon-button:hover {
    color: var(--accent-blue);
  }

  .filter-icon-button.active {
    color: var(--accent-blue);
  }

  .column-name {
    font-weight: 600;
    font-size: 12px;
  }

  .filter-input-wrapper {
    padding: 0.25rem 0;
    margin-top: 0.25rem;
  }

  .filter-input {
    width: 100%;
    padding: 0.25rem 0.5rem;
    font-size: 11px;
    border: 1px solid var(--grid-border);
    border-radius: 3px;
    background-color: var(--bg-input);
    color: var(--text-primary);
    transition: border-color 0.15s ease-in-out;
  }

  .filter-input:focus {
    outline: none;
    border-color: var(--accent-blue);
    box-shadow: 0 0 0 2px rgba(13, 110, 253, 0.15);
  }

  .filter-input:read-only {
    background-color: var(--grid-row-even);
    color: var(--text-muted);
    cursor: not-allowed;
    font-style: italic;
  }

  .filter-input.text-end {
    text-align: right;
  }

  .sort-button.numeric-sort {
    justify-content: flex-end;
  }

  .null-value {
    color: var(--text-muted);
  }

  .loading-indicator {
    position: sticky;
    left: 0;
  }
</style>
