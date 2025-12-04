<script>
  import { onMount } from "svelte";
  import { loadQueries, deleteQuery } from "../../core/integrations/tauri";
  import BasePanel from "../base/BasePanel.svelte";
  import PanelEmptyState from "../base/PanelEmptyState.svelte";
  import {
    formatTimestamp,
    showConfirmation,
    exportToJSON,
    filterItems,
  } from "../../shared/utils/ui/panelHelpers";

  export let onLoadQuery = () => {};

  let queries = [];
  let searchQuery = "";
  let loading = false;

  async function loadQueriesFromBackend() {
    loading = true;
    try {
      queries = await loadQueries();
    } catch (error) {
      console.error("Failed to load queries:", error);
      queries = [];
    }
    loading = false;
  }

  function handleLoadQuery(query) {
    onLoadQuery(query.content);
  }

  function handleDeleteQuery(id) {
    if (showConfirmation("Delete this saved query?")) {
      deleteQuery(id);
      loadQueriesFromBackend();
    }
  }

  function handleExportQueries() {
    try {
      exportToJSON(queries, "queries");
    } catch (error) {
      alert("Failed to export queries");
    }
  }

  onMount(() => {
    loadQueriesFromBackend();
  });

  $: filteredQueries = filterItems(queries, searchQuery, [
    "title",
    "content",
    "description",
  ]);
</script>

<BasePanel title="Saved Queries" icon="fas fa-floppy-disk" showToolbar={false}>
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
        <PanelEmptyState
          message={queries.length === 0
            ? "No saved queries yet"
            : "No matching queries"}
          icon="fa-floppy-disk"
          isSearchResult={queries.length > 0}
        />
      {:else}
        {#each filteredQueries as query (query.id)}
          <div class="panel-item">
            <div class="item-header">
              <button
                class="item-title-button"
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
            <div class="item-meta">
              <small>
                {formatTimestamp(query.created_at)} • {query.database_name ||
                  "N/A"}
              </small>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</BasePanel>

<style>
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

  .btn-delete:disabled,
  .btn-export:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .search-box {
    border-top: 1px solid var(--border-color);
    border-bottom: 1px solid var(--border-color);
  }
</style>
