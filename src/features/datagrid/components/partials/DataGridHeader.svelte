<script>
  export let finalQuery = "";
  export let executedQuery = "";
  export let viewMode = "grid";
  export let columnFilters = {};
  export let hasUnsavedEdits = false;
  export let editedRowsSize = 0;

  export let onViewModeToggle = null;
  export let onClearFilters = null;
  export let onSaveChanges = null;
  export let onCancelChanges = null;
</script>

<div class="d-flex align-items-center gap-2 p-2 data-header border-bottom">
  <!-- Query Display -->
  {#if finalQuery || executedQuery}
    <div
      class="d-flex align-items-center gap-2 font-monospace small flex-grow-1"
      style="min-width: 0;"
    >
      <div
        class="d-flex align-items-center gap-1 text-primary fw-semibold flex-shrink-0"
      >
        <i class="fas fa-code"></i>
        <span>Query:</span>
      </div>
      <div
        class="text-truncate query-display px-2 py-1 border rounded flex-grow-1"
        title={finalQuery || executedQuery}
        style="min-width: 0;"
      >
        {finalQuery || executedQuery}
      </div>
    </div>
  {/if}

  <!-- View Mode Toggle -->
  <div class="btn-group flex-shrink-0" role="group">
    <button
      type="button"
      class="btn btn-sm {viewMode === 'grid'
        ? 'btn-primary'
        : 'btn-outline-primary'}"
      on:click={() => viewMode === "json" && onViewModeToggle?.()}
      title="Grid View"
    >
      <i class="fas fa-table"></i> Grid
    </button>
    <button
      type="button"
      class="btn btn-sm {viewMode === 'json'
        ? 'btn-primary'
        : 'btn-outline-primary'}"
      on:click={() => viewMode === "grid" && onViewModeToggle?.()}
      title="JSON View"
    >
      <i class="fas fa-code"></i> JSON
    </button>
  </div>

  <!-- Clear Filters -->
  {#if Object.keys(columnFilters).length > 0}
    <button
      class="btn btn-sm btn-danger flex-shrink-0"
      on:click={() => onClearFilters?.()}
    >
      <i class="fas fa-times"></i> Clear filters
    </button>
  {/if}

  <!-- Unsaved Changes Indicator -->
  {#if hasUnsavedEdits && editedRowsSize > 0}
    <div class="ms-auto d-flex align-items-center gap-2">
      <span
        class="badge bg-warning text-dark"
        title={`${editedRowsSize} row(s) have unsaved changes`}
      >
        <i class="fas fa-pencil"></i>
        {editedRowsSize} unsaved
      </span>
      <button
        class="btn btn-sm btn-success"
        on:click={() => onSaveChanges?.()}
        title="Save changes"
      >
        <i class="fas fa-save"></i>
      </button>
      <button
        class="btn btn-sm btn-secondary"
        on:click={() => onCancelChanges?.()}
        title="Cancel changes"
      >
        <i class="fas fa-undo"></i>
      </button>
    </div>
  {/if}
</div>

<style>
  .data-header {
    background: var(--bg-tertiary);
  }

  .query-display {
    background: var(--bg-secondary);
    color: var(--text-primary);
    border-color: var(--border-color) !important;
  }
</style>
