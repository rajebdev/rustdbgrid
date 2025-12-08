<script>
  import QueryEditorModal from "../../modals/QueryEditorModal.svelte";

  export let finalQuery = "";
  export let executedQuery = "";
  export let viewMode = "grid";
  export let columnFilters = {};
  export let databaseType = null;

  export let onViewModeToggle = null;
  export let onClearFilters = null;
  export let onQueryEdit = null;

  let isQueryEditorOpen = false;

  function openQueryEditor() {
    console.log(
      "[DataGridHeader] openQueryEditor - finalQuery:",
      finalQuery ? finalQuery.substring(0, 50) + "..." : ""
    );
    console.log(
      "[DataGridHeader] openQueryEditor - executedQuery:",
      executedQuery ? executedQuery.substring(0, 50) + "..." : ""
    );
    console.log(
      "[DataGridHeader] openQueryEditor - databaseType:",
      databaseType
    );
    isQueryEditorOpen = true;
  }

  function closeQueryEditor() {
    isQueryEditorOpen = false;
  }

  function handleQuerySave(newQuery) {
    if (onQueryEdit) {
      onQueryEdit(newQuery);
    }
    closeQueryEditor();
  }

  function copyQuery() {
    const query = finalQuery || executedQuery;
    navigator.clipboard.writeText(query);
  }
</script>

<div class="d-flex align-items-center gap-2 p-2 data-header border-bottom">
  <!-- Query Display - always reserve space to prevent height jumping -->
  <div
    class="d-flex align-items-center gap-2 font-monospace small flex-grow-1"
    style="min-width: 0; min-height: 32px;"
  >
    {#if finalQuery || executedQuery}
      <div
        class="d-flex align-items-center gap-1 text-primary fw-semibold flex-shrink-0"
      >
        <i class="fas fa-code"></i>
        <span>Query:</span>
      </div>
      <div
        class="d-flex align-items-center gap-2 flex-grow-1 query-container"
        style="min-width: 0;"
      >
        <div
          class="text-truncate query-display px-2 py-1 border rounded flex-grow-1 cursor-pointer user-select-text"
          title={finalQuery || executedQuery}
          style="min-width: 0;"
          on:click={openQueryEditor}
          role="button"
          tabindex="0"
          on:keydown={(e) => e.key === "Enter" && openQueryEditor()}
        >
          {finalQuery || executedQuery}
        </div>
        <button
          class="btn btn-sm btn-outline-secondary flex-shrink-0"
          title="Copy query"
          on:click={copyQuery}
        >
          <i class="fas fa-copy"></i>
        </button>
        <button
          class="btn btn-sm btn-outline-warning flex-shrink-0"
          title="Edit query"
          on:click={openQueryEditor}
        >
          <i class="fas fa-edit"></i>
        </button>
      </div>
    {/if}
  </div>

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
</div>

<!-- Query Editor Modal -->
<QueryEditorModal
  show={isQueryEditorOpen}
  query={finalQuery || executedQuery}
  {databaseType}
  onClose={closeQueryEditor}
  onSave={handleQuerySave}
/>

<style>
  .data-header {
    background: var(--bg-tertiary);
  }

  .query-display {
    background: var(--bg-secondary);
    color: var(--text-primary);
    border-color: var(--border-color) !important;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .query-display:hover {
    border-color: var(--text-primary) !important;
    background: var(--bg-secondary);
    opacity: 0.9;
  }

  .query-container {
    position: relative;
  }

  :global(.user-select-text) {
    user-select: text;
  }

  :global(.cursor-pointer) {
    cursor: pointer;
  }
</style>
