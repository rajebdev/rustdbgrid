<script>
  export let value = [];
  export let maxPreviewItems = 3;

  let expanded = false;

  function toggleExpand(e) {
    e.stopPropagation();
    expanded = !expanded;
  }

  function formatItem(item) {
    if (item === null) return "NULL";
    if (typeof item === "object") return JSON.stringify(item);
    return String(item);
  }

  $: previewText =
    value.length === 0
      ? "[]"
      : value.length <= maxPreviewItems
        ? `[${value.map(formatItem).join(", ")}]`
        : `[${value.slice(0, maxPreviewItems).map(formatItem).join(", ")}, ...]`;
</script>

<div class="array-cell">
  {#if value.length === 0}
    <span class="text-muted">[]</span>
  {:else if value.length <= maxPreviewItems && !value.some((v) => typeof v === "object" && v !== null)}
    <!-- Simple array, show inline -->
    <span class="array-inline">[{value.map(formatItem).join(", ")}]</span>
  {:else}
    <!-- Complex or long array, show expandable -->
    <button
      class="btn btn-link btn-sm p-0 array-toggle"
      on:click={toggleExpand}
      title={expanded ? "Collapse array" : "Expand array"}
    >
      <i class="fas fa-caret-{expanded ? 'down' : 'right'} me-1"></i>
      <span class="badge bg-secondary me-1">{value.length}</span>
      {#if !expanded}
        <span class="array-preview text-truncate">{previewText}</span>
      {/if}
    </button>

    {#if expanded}
      <div class="array-items mt-1">
        {#each value as item, i}
          <div class="array-item">
            <span class="array-index text-muted">[{i}]</span>
            <span
              class="array-value {item === null ? 'text-muted fst-italic' : ''}"
            >
              {formatItem(item)}
            </span>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  .array-cell {
    max-width: 100%;
  }

  .array-inline {
    font-family: var(--bs-font-monospace);
    font-size: 0.85em;
  }

  .array-toggle {
    text-decoration: none !important;
    color: inherit !important;
    display: inline-flex;
    align-items: center;
    max-width: 100%;
  }

  .array-toggle:hover {
    color: var(--bs-primary) !important;
  }

  .array-preview {
    font-family: var(--bs-font-monospace);
    font-size: 0.85em;
    color: var(--bs-secondary);
    max-width: 200px;
    display: inline-block;
    vertical-align: middle;
  }

  .array-items {
    border-left: 2px solid var(--bs-border-color);
    padding-left: 8px;
    margin-left: 4px;
    max-height: 200px;
    overflow-y: auto;
  }

  .array-item {
    font-family: var(--bs-font-monospace);
    font-size: 0.85em;
    padding: 2px 0;
    display: flex;
    gap: 8px;
  }

  .array-index {
    min-width: 30px;
    text-align: right;
    font-size: 0.8em;
  }

  .array-value {
    word-break: break-all;
  }
</style>
