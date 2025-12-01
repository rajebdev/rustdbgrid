<script>
  import { onMount } from "svelte";
  import { loadQueries, deleteQuery } from "../../utils/tauri";

  export let onLoadQuery = () => {};

  let queries = [];
  let searchQuery = "";
  let filteredQueries = [];
  let loading = false;

  async function loadQueriesFromBackend() {
    loading = true;
    try {
      queries = await loadQueries();
      updateFiltered();
    } catch (error) {
      console.error("Failed to load queries:", error);
      queries = [];
    }
    loading = false;
  }

  function updateFiltered() {
    if (searchQuery.trim()) {
      filteredQueries = queries.filter(
        (q) =>
          q.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
          q.content.toLowerCase().includes(searchQuery.toLowerCase()) ||
          q.description.toLowerCase().includes(searchQuery.toLowerCase())
      );
    } else {
      filteredQueries = queries;
    }
  }

  function handleLoadQuery(query) {
    onLoadQuery(query.content);
  }

  function handleDeleteQuery(id) {
    if (confirm("Delete this saved query?")) {
      deleteQuery(id);
      loadQueriesFromBackend();
    }
  }

  function handleExportQueries() {
    try {
      const dataStr = JSON.stringify(queries, null, 2);
      const dataBlob = new Blob([dataStr], { type: "application/json" });
      const url = URL.createObjectURL(dataBlob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `queries_${new Date().toISOString().split("T")[0]}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error("Failed to export queries:", error);
    }
  }

  function formatDate(dateString) {
    const timestamp =
      typeof dateString === "number" ? dateString * 1000 : dateString;
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now - date;
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return "just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;

    return date.toLocaleDateString();
  }

  onMount(() => {
    loadQueriesFromBackend();
  });

  $: searchQuery, updateFiltered();
</script>

<div class="query-recovery-panel h-100 d-flex flex-column">
  <div class="panel-header border-bottom p-2">
    <h6 class="mb-0">
      <i class="fas fa-floppy-disk"></i> Saved Queries
    </h6>
  </div>

  <div class="flex-grow-1 overflow-auto">
    <div class="section">
      <div class="section-header">
        <div class="section-title">
          <small>All Queries</small>
        </div>
        <button
          class="btn-export"
          on:click={handleExportQueries}
          disabled={queries.length === 0}
          title="Export all queries"
        >
          <i class="fas fa-download"></i>
        </button>
      </div>

      <div class="search-box p-2">
        <input
          type="text"
          class="form-control form-control-sm"
          placeholder="Search..."
          bind:value={searchQuery}
        />
      </div>

      <div class="items-list">
        {#if loading}
          <div class="text-muted text-center p-3">
            <small><i class="fas fa-spinner fa-spin"></i> Loading...</small>
          </div>
        {:else if filteredQueries.length === 0}
          <div class="text-muted text-center p-3">
            {#if queries.length === 0}
              <small>No saved queries yet</small>
            {:else}
              <small>No matching queries</small>
            {/if}
          </div>
        {:else}
          {#each filteredQueries as query (query.id)}
            <div class="query-item">
              <div class="item-header">
                <button
                  class="btn-load-query"
                  on:click={() => handleLoadQuery(query)}
                  title={query.content}
                >
                  <strong>{query.title}</strong>
                </button>
                <button
                  class="btn-delete"
                  on:click={() => handleDeleteQuery(query.id)}
                  title="Delete"
                >
                  <i class="fas fa-trash-alt"></i>
                </button>
              </div>
              {#if query.description}
                <small class="text-muted d-block mt-1">
                  {query.description}
                </small>
              {/if}
              <small class="text-muted d-block mt-1">
                {formatDate(query.created_at)} • {query.database_name || "N/A"}
              </small>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .query-recovery-panel {
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .panel-header {
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  .panel-header h6 {
    margin: 0;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  .section {
    border-bottom: 1px solid var(--border-color);
    padding: 12px 0;
  }

  .section:last-child {
    border-bottom: none;
  }

  .section-title {
    padding: 0 12px 8px 12px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 12px 8px 12px;
  }

  .items-list {
    display: flex;
    flex-direction: column;
  }

  .query-item {
    padding: 8px 12px;
    border-bottom: 1px solid var(--bg-tertiary);
    transition: background 0.15s ease;
  }

  .query-item:hover {
    background: var(--bg-tertiary);
  }

  .item-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 8px;
  }

  .btn-load-query {
    background: none;
    border: none;
    padding: 0;
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
    flex-grow: 1;
    transition: color 0.15s ease;
  }

  .btn-load-query:hover {
    color: var(--accent-blue);
  }

  .btn-delete,
  .btn-export {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0 4px;
    font-size: 11px;
    transition: color 0.15s ease;
  }

  .btn-delete:hover,
  .btn-export:hover {
    color: var(--danger);
  }

  .search-box {
    border-top: 1px solid var(--border-color);
    border-bottom: 1px solid var(--border-color);
  }
</style>
