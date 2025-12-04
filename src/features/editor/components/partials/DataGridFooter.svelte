<script>
  export let totalRows = 0;
  export let displayRowsLength = 0;
  export let columnCount = 0;
  export let executionTime = "0";
  export let displayData = null;
</script>

{#if displayData}
  <div
    class="sticky-bottom data-footer border-top shadow-sm"
    style="position: sticky; bottom: 0; z-index: 10;"
  >
    <div class="d-flex align-items-center justify-content-end gap-2 p-2">
      <span class="badge bg-primary">
        <i class="fas fa-table"></i>
        {#if totalRows > displayRowsLength}
          Rows: {displayRowsLength.toLocaleString()} of {totalRows.toLocaleString()}
        {:else}
          Rows: {displayRowsLength.toLocaleString()}
        {/if}
      </span>
      <span class="badge bg-info">
        <i class="fas fa-columns"></i> Columns: {columnCount}
      </span>
      <span class="badge bg-secondary">
        <i class="fas fa-clock"></i>
        {executionTime}ms
      </span>
    </div>
  </div>
{/if}

<style>
  .data-footer {
    background: var(--bg-tertiary);
  }
</style>
