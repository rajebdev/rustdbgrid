<script>
  import { onMount } from "svelte";
  import { createEventDispatcher } from "svelte";
  import { queryListStore } from "../../stores/queryList";
  import {
    loadQueriesFromFolder,
    deleteQueryFile,
  } from "../../services/queryFileService";

  const dispatch = createEventDispatcher();

  export let show = false;

  let searchQuery = "";
  let queries = [];
  let loading = true;

  // Load queries when modal is shown
  $: if (show) {
    loadQueries();
  }

  async function loadQueries() {
    loading = true;
    try {
      // Load queries from folder
      const fileQueries = await loadQueriesFromFolder();

      // Merge with saved queries from localStorage (if any)
      const savedQueries = $queryListStore;

      // Combine both, preferring file-based queries
      queries = [...fileQueries, ...savedQueries];
    } catch (error) {
      console.error("Failed to load queries:", error);
      queries = [];
    } finally {
      loading = false;
    }
  }

  $: filteredQueries = queries.filter((query) => {
    const search = searchQuery.toLowerCase();
    return (
      query.title.toLowerCase().includes(search) ||
      query.content.toLowerCase().includes(search) ||
      (query.description && query.description.toLowerCase().includes(search))
    );
  });

  function close() {
    dispatch("close");
  }

  function openQuery(query) {
    dispatch("open", query);
    close();
  }

  async function deleteQuery(query, event) {
    event.stopPropagation();
    if (!confirm("Are you sure you want to delete this query?")) {
      return;
    }

    try {
      if (
        (query.isFile || query.is_file) &&
        (query.filePath || query.file_path)
      ) {
        // Delete file-based query
        await deleteQueryFile(query.filePath || query.file_path);
      } else {
        // Delete localStorage query
        queryListStore.deleteQuery(query.id);
      }

      // Reload queries
      await loadQueries();
    } catch (error) {
      console.error("Failed to delete query:", error);
      alert("Failed to delete query: " + error);
    }
  }

  function formatDate(dateString) {
    const date = new Date(dateString);
    return date.toLocaleDateString() + " " + date.toLocaleTimeString();
  }

  function handleKeydown(event) {
    if (event.key === "Escape") {
      close();
    }
  }

  function focusSearch(node) {
    node.focus();
  }
</script>

{#if show}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="modal-backdrop show" on:click={close}></div>
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
  <div
    class="modal d-block"
    tabindex="-1"
    on:keydown={handleKeydown}
    role="dialog"
  >
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      class="modal-dialog modal-dialog-centered modal-lg"
      on:click|stopPropagation
    >
      <div class="modal-content">
        <div class="modal-header bg-primary text-white">
          <h5 class="modal-title">
            <i class="fas fa-list"></i> Saved Queries
          </h5>
          <button
            type="button"
            class="btn-close btn-close-white"
            on:click={close}
          ></button>
        </div>

        <div class="modal-body">
          <div class="search-box mb-3">
            <i class="fas fa-search search-icon"></i>
            <input
              type="text"
              class="form-control"
              placeholder="Search queries by title, content, or description..."
              bind:value={searchQuery}
              use:focusSearch
            />
          </div>

          {#if loading}
            <div class="loading-state">
              <i class="fas fa-spinner fa-spin"></i>
              <p>Loading queries...</p>
            </div>
          {:else if filteredQueries.length > 0}
            <div class="query-list">
              {#each filteredQueries as query (query.id)}
                <div
                  class="query-item"
                  on:click={() => openQuery(query)}
                  on:keydown={(e) => e.key === "Enter" && openQuery(query)}
                  role="button"
                  tabindex="0"
                >
                  <div class="query-header">
                    <div class="query-title">
                      <i class="fas fa-file-code"></i>
                      {query.title}
                      {#if query.isFile || query.is_file}
                        <span class="badge bg-secondary ms-2">File</span>
                      {/if}
                    </div>
                    <button
                      class="btn btn-sm btn-danger btn-delete"
                      on:click={(e) => deleteQuery(query, e)}
                      title="Delete query"
                    >
                      <i class="fas fa-trash"></i>
                    </button>
                  </div>
                  {#if query.description}
                    <div class="query-description">{query.description}</div>
                  {/if}
                  <div class="query-preview">
                    <code
                      >{query.content.substring(0, 150)}{query.content.length >
                      150
                        ? "..."
                        : ""}</code
                    >
                  </div>
                  <div class="query-footer">
                    {#if query.createdAt || query.created_at}
                      <span class="query-date">
                        <i class="fas fa-clock"></i>
                        Created: {formatDate(
                          query.createdAt || query.created_at
                        )}
                      </span>
                    {/if}
                    {#if query.lastModified || query.last_modified}
                      <span class="query-date">
                        <i class="fas fa-edit"></i>
                        Modified: {formatDate(
                          query.lastModified || query.last_modified
                        )}
                      </span>
                    {:else if query.lastUsed}
                      <span class="query-date">
                        <i class="fas fa-history"></i>
                        Last used: {formatDate(query.lastUsed)}
                      </span>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {:else if searchQuery}
            <div class="empty-state">
              <i class="fas fa-search"></i>
              <p>No queries found matching "{searchQuery}"</p>
            </div>
          {:else}
            <div class="empty-state">
              <i class="fas fa-inbox"></i>
              <p>No saved queries yet</p>
              <small>Save queries from the Query List panel</small>
            </div>
          {/if}
        </div>

        <div class="modal-footer">
          <div class="query-count">
            {filteredQueries.length}
            {filteredQueries.length === 1 ? "query" : "queries"}
          </div>
          <button type="button" class="btn btn-secondary" on:click={close}>
            <i class="fas fa-times"></i> Close
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .search-box {
    position: relative;
  }

  .search-icon {
    position: absolute;
    left: 12px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-muted);
    font-size: 14px;
  }

  .search-box input {
    padding-left: 36px;
  }

  .query-list {
    max-height: 500px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .query-item {
    padding: 16px;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    background: var(--bg-tertiary);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .query-item:hover {
    background: var(--hover-bg);
    border-color: var(--accent-blue);
    transform: translateY(-2px);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .query-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }

  .query-title {
    font-weight: 600;
    font-size: 15px;
    color: var(--text-primary);
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .query-title i {
    color: var(--accent-blue);
  }

  .btn-delete {
    opacity: 0;
    transition: opacity 0.2s ease;
    padding: 4px 8px;
    font-size: 12px;
  }

  .query-item:hover .btn-delete {
    opacity: 1;
  }

  .query-description {
    font-size: 13px;
    color: var(--text-secondary);
    margin-bottom: 8px;
    font-style: italic;
  }

  .query-preview {
    background: var(--bg-secondary);
    padding: 8px 12px;
    border-radius: 4px;
    margin-bottom: 8px;
    font-family: "Consolas", "Monaco", monospace;
    font-size: 12px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .query-footer {
    display: flex;
    gap: 16px;
    font-size: 11px;
    color: var(--text-muted);
  }

  .query-date {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 20px;
    color: var(--text-muted);
    text-align: center;
  }

  .empty-state i {
    font-size: 48px;
    margin-bottom: 16px;
    opacity: 0.5;
  }

  .empty-state p {
    margin: 0;
    font-size: 16px;
    color: var(--text-secondary);
  }

  .empty-state small {
    margin-top: 8px;
    font-size: 13px;
  }

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 20px;
    color: var(--text-muted);
    text-align: center;
  }

  .loading-state i {
    font-size: 48px;
    margin-bottom: 16px;
    color: var(--accent-blue);
  }

  .loading-state p {
    margin: 0;
    font-size: 16px;
    color: var(--text-secondary);
  }

  .query-count {
    flex: 1;
    text-align: left;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .modal-lg {
    max-width: 800px;
  }
</style>
