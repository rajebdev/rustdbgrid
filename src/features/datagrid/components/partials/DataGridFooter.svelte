<script>
  import SavePreviewModal from "../../modals/SavePreviewModal.svelte";
  import {
    addNewRow,
    deleteRow,
    duplicateRow,
    getColumnsFromData,
  } from "../../services/gridRowService";

  export let displayRowsLength = 0;
  export let displayData = null;
  export let onCancelChanges = null;
  export let connectionId = null;
  export let database = null;
  export let table = null;
  export let schema = null;
  export let newRows = new Map();
  export let editedRows = new Map();
  export let deletedRows = new Set();
  export let displayRows = [];
  export let onRefreshData = null;
  export let onEditCell = null;
  export let columns = []; // Column definitions
  export let selectedRows = new Set(); // Selected row indices
  export let onDisplayDataChange = null; // Callback when display data changes
  export let selectedCell = null; // Selected cell { rowIndex, column }
  export let originalRowData = new Map(); // Original row data for tracking changes
  export let paginateLimit = 200; // Rows per page limit
  export let onPaginateLimitChange = null; // Callback when limit changes
  export let executionTime = 0; // Total time from request start to render (ms)
  export let fetchTime = 0; // BE fetch time only (ms)
  export let onSetupAutoRefresh = null; // Callback to setup auto-refresh (managed by parent)
  export let onStopAutoRefresh = null; // Callback to stop auto-refresh (managed by parent)
  export let isAutoRefreshActive = false; // Auto-refresh state from parent

  let localPaginateLimit = paginateLimit; // Local copy to track changes

  // Sync local copy when prop changes from parent
  $: localPaginateLimit = paginateLimit;

  let refreshDropdownOpen = false;
  let saveDropdownOpen = false;
  let lastFetchTime = new Date().toLocaleString();
  let dropdownContainer;
  let showSavePreviewModal = false;

  // Check if there are any changes
  $: hasChanges =
    newRows.size > 0 || editedRows.size > 0 || deletedRows.size > 0;

  // Format timing display
  $: executionTimeDisplay =
    executionTime > 0 ? (executionTime / 1000).toFixed(1) : "0.0";
  $: fetchTimeDisplay = fetchTime > 0 ? (fetchTime / 1000).toFixed(1) : "0.0";

  const handleRefresh = (type) => {
    if (type === "instant") {
      // Immediate refresh
      if (onRefreshData) {
        onRefreshData();
      }
    } else if (type === "custom") {
      // Open settings for custom interval (future enhancement)
      console.log("Custom refresh interval - to be implemented");
    } else {
      // Parse interval (e.g., "5s" -> 5000ms)
      const match = type.match(/(\d+)s/);
      if (match) {
        const intervalMs = parseInt(match[1]) * 1000;
        if (onSetupAutoRefresh) {
          onSetupAutoRefresh(intervalMs);
        }
        console.log(`🔄 Auto-refresh enabled every ${match[1]}s`);
      }
    }

    closeDropdowns();
  };

  const handleSave = (type) => {
    if (type === "instant") {
      showSavePreviewModal = true;
    } else if (type === "generateScript") {
      // Just open modal, the modal will handle script generation
      showSavePreviewModal = true;
    } else if (type === "instantConfirm") {
      // With confirmation
      showSavePreviewModal = true;
    }
    closeDropdowns();
  };

  const handleEditCell = () => {
    if (!selectedCell) {
      console.warn("⚠️ Please select a cell to edit");
      closeDropdowns();
      return;
    }

    if (onEditCell) {
      onEditCell();
    }
    closeDropdowns();
  };

  const handleAddRow = () => {
    if (!displayData) return;

    const cols =
      columns.length > 0
        ? columns
        : getColumnsFromData(displayData.rows || displayData);

    // Determine insert position: prefer selectedCell, then selectedRows, otherwise insert at middle of visible rows
    let insertAfterIndex = null;

    if (selectedCell?.rowIndex !== undefined) {
      // If a cell is selected, insert after that row
      insertAfterIndex = selectedCell.rowIndex;
    } else if (selectedRows && selectedRows.size > 0) {
      // If rows are selected, insert after the last selected row
      insertAfterIndex = Math.max(...Array.from(selectedRows));
    } else {
      // No selection: insert at middle of displayRows (which shows visible rows)
      // This way the new row appears in the middle of current viewport
      if (displayRows && displayRows.length > 0) {
        const middleIndex = Math.floor(displayRows.length / 2);
        // Find the actual index in displayData based on displayRows
        const middleRow = displayRows[middleIndex];
        if (middleRow) {
          insertAfterIndex = displayData.rows.indexOf(middleRow);
        }
      }
    }

    // Pass displayData.rows (the array) to addNewRow, not the whole displayData object
    const rows = displayData.rows || displayData;
    const result = addNewRow(rows, cols, newRows, insertAfterIndex);

    // Trigger callback with the new rows array
    if (onDisplayDataChange) {
      onDisplayDataChange(result.displayData);
    }

    newRows = result.newRows;

    console.log(
      "✅ New row added" +
        (insertAfterIndex !== null
          ? ` after row ${insertAfterIndex + 1}`
          : " at end")
    );
    closeDropdowns();
  };

  const handleDuplicateRow = () => {
    if (!displayData) {
      console.warn("⚠️ No data to duplicate");
      return;
    }

    // Determine which row to duplicate: prefer selectedCell, then selectedRows
    let rowIndex = null;

    if (selectedCell?.rowIndex !== undefined) {
      // If a cell is selected, duplicate that row
      rowIndex = selectedCell.rowIndex;
    } else if (selectedRows && selectedRows.size > 0) {
      // If rows are selected, duplicate the first selected row
      rowIndex = Array.from(selectedRows)[0];
    } else {
      console.warn("⚠️ Please select a cell or row to duplicate");
      return;
    }

    // Pass displayData.rows (the array) to duplicateRow with columns
    const rows = displayData.rows || displayData;
    const result = duplicateRow(rows, rowIndex, newRows, editedRows, columns);

    newRows = result.newRows;

    // Trigger callback with the new rows array
    if (onDisplayDataChange) {
      onDisplayDataChange(result.displayData);
    }

    console.log("✅ Row duplicated from row", rowIndex + 1);
    closeDropdowns();
  };

  const handleDeleteRow = () => {
    if (!displayData) {
      console.warn("⚠️ No data to delete");
      return;
    }

    // Determine which row to delete: prefer selectedCell, then selectedRows
    let rowIndex = null;

    if (selectedCell?.rowIndex !== undefined) {
      // If a cell is selected, delete that row
      rowIndex = selectedCell.rowIndex;
    } else if (selectedRows && selectedRows.size > 0) {
      // If rows are selected, delete the first selected row
      rowIndex = Array.from(selectedRows)[0];
    } else {
      console.warn("⚠️ Please select a cell or row to delete");
      return;
    }

    // Pass displayData.rows (the array) to deleteRow with columns
    const rows = displayData.rows || displayData;
    const result = deleteRow(
      rows,
      rowIndex,
      deletedRows,
      editedRows,
      newRows,
      columns
    );

    deletedRows = result.deletedRows;
    editedRows = result.editedRows;
    newRows = result.newRows;
    selectedRows.clear(); // Clear selection after delete
    selectedRows = selectedRows;
    selectedCell = null; // Clear cell selection after delete

    // Trigger callback with the new rows array
    if (onDisplayDataChange) {
      onDisplayDataChange(result.displayData);
    }

    console.log("✅ Row marked for deletion");
    closeDropdowns();
  };

  const handleCancel = () => {
    if (onCancelChanges) {
      onCancelChanges();
    }
    closeDropdowns();
  };

  const handlePaginateLimitChange = (e) => {
    if (e.key === "Enter") {
      // Get value directly from input element (not from bound state)
      let newLimit = parseInt(e.target.value);
      if (isNaN(newLimit) || newLimit < 1) {
        newLimit = 200; // Reset to default if invalid
        console.warn("⚠️ Invalid limit value, resetting to default: 200");
      }
      localPaginateLimit = newLimit;
      console.log("📄 Paginate limit changed to:", newLimit);
      // Notify parent component of the change
      if (onPaginateLimitChange) {
        onPaginateLimitChange(newLimit);
      }
    }
  };

  const handlePaginateLimitBlur = (e) => {
    // Also validate on blur (clicking outside)
    let newLimit = parseInt(e.target.value);
    if (isNaN(newLimit) || newLimit < 1) {
      newLimit = 200;
      localPaginateLimit = newLimit;
      console.warn("⚠️ Invalid limit value on blur, resetting to default: 200");
    } else {
      // Value is valid, check if it changed
      if (newLimit !== paginateLimit) {
        localPaginateLimit = newLimit;
        console.log("📄 Paginate limit changed to:", newLimit);
      }
    }
    // Always notify parent of the final value
    if (onPaginateLimitChange) {
      onPaginateLimitChange(localPaginateLimit);
    }
  };

  const handleClickOutside = (e) => {
    if (dropdownContainer && !dropdownContainer.contains(e.target)) {
      refreshDropdownOpen = false;
      saveDropdownOpen = false;
    }
  };

  const closeDropdowns = () => {
    refreshDropdownOpen = false;
    saveDropdownOpen = false;
  };

  const handleStopRefresh = () => {
    if (onStopAutoRefresh) {
      onStopAutoRefresh();
    }
    console.log("⏹️ Auto-refresh stopped");
    closeDropdowns();
  };

  const handleSaveSuccess = (response) => {
    console.log("✅ Save successful!", response);
    showSavePreviewModal = false;
    // Clear all changes
    newRows.clear();
    editedRows.clear();
    deletedRows.clear();
    newRows = newRows;
    editedRows = editedRows;
    deletedRows = deletedRows;
    // Refresh data
    if (onRefreshData) {
      onRefreshData();
    }
  };
</script>

<svelte:window on:click={handleClickOutside} />

{#if displayData}
  <div
    class="sticky-bottom data-footer border-top shadow-sm"
    style="position: sticky; bottom: 0; z-index: 20;"
  >
    <div
      class="d-flex align-items-center justify-content-between gap-3 p-2"
      bind:this={dropdownContainer}
    >
      <!-- Left Toolbar -->
      <div class="d-flex align-items-center gap-2">
        <!-- Refresh Button -->
        <div class="dropdown-up">
          {#if refreshDropdownOpen}
            <div class="dropdown-menu-up show">
              <button
                class="dropdown-item"
                on:click={() => handleRefresh("instant")}
              >
                <i class="fas fa-bolt"></i> Instant
              </button>
              <button
                class="dropdown-item"
                on:click={() => handleRefresh("1s")}
              >
                Every 1s
              </button>
              <button
                class="dropdown-item"
                on:click={() => handleRefresh("5s")}
              >
                Every 5s
              </button>
              <button
                class="dropdown-item"
                on:click={() => handleRefresh("10s")}
              >
                Every 10s
              </button>
              <button
                class="dropdown-item"
                on:click={() => handleRefresh("15s")}
              >
                Every 15s
              </button>
              <button
                class="dropdown-item"
                on:click={() => handleRefresh("30s")}
              >
                Every 30s
              </button>
              <button
                class="dropdown-item"
                on:click={() => handleRefresh("60s")}
              >
                Every 60s
              </button>
              <div class="dropdown-divider"></div>
              <button
                class="dropdown-item"
                on:click={() => handleRefresh("custom")}
              >
                <i class="fas fa-cog"></i> Custom
              </button>
            </div>
          {/if}
          <div class="btn-group-refresh">
            <button
              class="btn btn-sm {isAutoRefreshActive
                ? 'btn-outline-danger'
                : 'btn-outline-primary'} d-flex align-items-center gap-2"
              title={isAutoRefreshActive ? "Stop Auto-Refresh" : "Refresh Data"}
              on:click={() => {
                if (isAutoRefreshActive) {
                  handleStopRefresh();
                } else {
                  handleRefresh("instant");
                }
              }}
            >
              <i
                class={isAutoRefreshActive
                  ? "fas fa-stop-circle"
                  : "fas fa-sync-alt"}
              ></i>
              <span>{isAutoRefreshActive ? "Stop" : "Refresh"}</span>
            </button>
            <button
              class="btn btn-sm {isAutoRefreshActive
                ? 'btn-outline-danger'
                : 'btn-outline-primary'}"
              on:click={() => {
                refreshDropdownOpen = !refreshDropdownOpen;
                if (refreshDropdownOpen) saveDropdownOpen = false;
              }}
            >
              <i class="fas fa-chevron-up" style="font-size: 0.65rem;"></i>
            </button>
          </div>
        </div>

        <!-- Save Button -->
        <div class="dropdown-up">
          {#if saveDropdownOpen}
            <div class="dropdown-menu-up show">
              <button
                class="dropdown-item"
                on:click={() => handleSave("generateScript")}
              >
                <i class="fas fa-file-code"></i> Generate Script
              </button>
              <button
                class="dropdown-item"
                on:click={() => handleSave("instant")}
              >
                <i class="fas fa-bolt"></i> Instant
              </button>
              <button
                class="dropdown-item"
                on:click={() => handleSave("instantConfirm")}
              >
                <i class="fas fa-check-circle"></i> Instant with Confirmation
              </button>
            </div>
          {/if}
          <div class="btn-group-save">
            <button
              class="btn btn-sm d-flex align-items-center gap-2 {hasChanges
                ? 'btn-outline-success'
                : 'btn-outline-secondary'}"
              title="Save Data"
              disabled={!hasChanges}
              on:click={() => handleSave("instant")}
            >
              <i class="fas fa-save"></i>
              <span>Save</span>
            </button>
            <button
              class="btn btn-sm {hasChanges
                ? 'btn-outline-success'
                : 'btn-outline-secondary'}"
              disabled={!hasChanges}
              on:click={() => {
                saveDropdownOpen = !saveDropdownOpen;
                if (saveDropdownOpen) refreshDropdownOpen = false;
              }}
            >
              <i class="fas fa-chevron-up" style="font-size: 0.65rem;"></i>
            </button>
          </div>
        </div>

        <!-- Cancel Button -->
        <button
          class="btn btn-sm d-flex align-items-center gap-2 {hasChanges
            ? 'btn-outline-danger'
            : 'btn-outline-secondary'}"
          title="Cancel Changes"
          disabled={!hasChanges}
          on:click={handleCancel}
        >
          <i class="fas fa-times"></i>
          <span>Cancel</span>
        </button>

        <div class="vr"></div>

        <!-- Edit Cell -->
        <button
          class="btn btn-sm {selectedCell
            ? 'btn-outline-warning'
            : 'btn-outline-secondary'}"
          title="Edit Selected Cell (or double-click)"
          disabled={!selectedCell}
          on:click={handleEditCell}
        >
          <i class="fas fa-edit"></i>
        </button>

        <!-- Add Row -->
        <button
          class="btn btn-sm btn-outline-success"
          title="Add Row"
          on:click={handleAddRow}
        >
          <i class="fas fa-plus"></i>
        </button>

        <!-- Duplicate Row -->
        <button
          class="btn btn-sm {selectedCell || selectedRows.size > 0
            ? 'btn-outline-primary'
            : 'btn-outline-secondary'}"
          title="Duplicate Row"
          disabled={!selectedCell &&
            (selectedRows.size === 0 || selectedRows === null)}
          on:click={handleDuplicateRow}
        >
          <i class="fas fa-copy"></i>
        </button>

        <!-- Delete Row -->
        <button
          class="btn btn-sm {selectedCell || selectedRows.size > 0
            ? 'btn-outline-danger'
            : 'btn-outline-secondary'}"
          title="Delete Current Row"
          disabled={!selectedCell &&
            (selectedRows.size === 0 || selectedRows === null)}
          on:click={handleDeleteRow}
        >
          <i class="fas fa-trash"></i>
        </button>

        <div class="vr"></div>

        <!-- Paginate Limit -->
        <div class="d-flex align-items-center gap-2">
          <label for="paginate-limit" class="mb-0" style="font-size: 0.85rem;"
            >Limit:</label
          >
          <input
            id="paginate-limit"
            type="number"
            class="form-control form-control-sm"
            style="width: 70px;"
            bind:value={localPaginateLimit}
            on:keydown={handlePaginateLimitChange}
            on:blur={handlePaginateLimitBlur}
            min="1"
          />
        </div>
      </div>

      <!-- Right Detail Info -->
      <div class="text-muted" style="font-size: 0.85rem;">
        {displayRowsLength.toLocaleString()} rows fetched - {executionTimeDisplay}s
        ({fetchTimeDisplay}s fetch), pada {lastFetchTime}
      </div>
    </div>
  </div>
{/if}

<SavePreviewModal
  isOpen={showSavePreviewModal}
  {connectionId}
  {database}
  {table}
  {schema}
  {newRows}
  {editedRows}
  {deletedRows}
  {displayRows}
  {originalRowData}
  {columns}
  onClose={() => (showSavePreviewModal = false)}
  onSaveSuccess={handleSaveSuccess}
/>

<style>
  .data-footer {
    background: var(--bg-tertiary);
  }

  .dropdown-up {
    position: relative;
  }

  .btn-group-refresh,
  .btn-group-save {
    display: flex;
    gap: 0;
  }

  .btn-group-refresh .btn:first-child,
  .btn-group-save .btn:first-child {
    border-top-right-radius: 0;
    border-bottom-right-radius: 0;
    border-right: none;
  }

  .btn-group-refresh .btn:last-child,
  .btn-group-save .btn:last-child {
    border-top-left-radius: 0;
    border-bottom-left-radius: 0;
  }

  /* Disabled button styling */
  button:disabled {
    opacity: 0.5 !important;
    cursor: not-allowed !important;
  }

  .dropdown-menu-up {
    position: absolute;
    bottom: 100%;
    left: 0;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    margin-bottom: 2px;
    min-width: 150px;
    box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.15);
  }

  .dropdown-item {
    display: block;
    width: 100%;
    padding: 0.5rem 1rem;
    border: none;
    background: none;
    text-align: left;
    cursor: pointer;
    font-size: 0.875rem;
    color: inherit;
  }

  .dropdown-item:hover {
    background: var(--bg-tertiary);
  }

  .dropdown-divider {
    margin: 0.5rem 0;
    border-top: 1px solid var(--border-color);
  }

  /* Pagination limit input styling with dark mode support */
  #paginate-limit {
    background-color: var(--bg-input, #ffffff);
    color: var(--text-primary, #000000);
    border-color: var(--border-color, #cccccc);
    text-align: right;
  }

  #paginate-limit:focus {
    background-color: var(--bg-input, #ffffff);
    color: var(--text-primary, #000000);
    border-color: var(--accent-blue, #0d6efd);
    box-shadow: 0 0 0 0.2rem rgba(13, 110, 253, 0.25);
  }

  /* Style spinner buttons (arrow up/down) */
  #paginate-limit::-webkit-outer-spin-button,
  #paginate-limit::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
    display: none;
  }

  /* Firefox spinner styling */
  #paginate-limit {
    appearance: textfield;
  }
</style>
