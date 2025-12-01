<script>
  import { queryHistoryStore } from "../../stores/queryHistory";

  let searchTerm = "";

  function handleDelete(id) {
    if (confirm("Delete this history entry?")) {
      queryHistoryStore.deleteEntry(id);
    }
  }

  function handleLoadQuery(query) {
    // Dispatch event to parent/editor
    window.dispatchEvent(new CustomEvent("load-query", { detail: { query } }));
  }

  function handleClearHistory() {
    if (confirm("Clear all history?")) {
      queryHistoryStore.clearHistory();
    }
  }

  $: filteredHistory = $queryHistoryStore.filter(
    (h) =>
      h.query.toLowerCase().includes(searchTerm.toLowerCase()) ||
      h.databaseName.toLowerCase().includes(searchTerm.toLowerCase())
  );
</script>

<div class="query-history-panel">
  <div class="panel-header">
    <h6 class="mb-0">
      <i class="fas fa-history"></i> Query History
    </h6>
  </div>

  <div class="panel-toolbar">
    <input
      type="text"
      class="form-control form-control-sm"
      placeholder="Search history..."
      bind:value={searchTerm}
    />
    <button
      class="btn btn-sm btn-outline-danger"
      on:click={handleClearHistory}
      title="Clear history"
    >
      <i class="fas fa-times"></i>
    </button>
  </div>

  <div class="history-list">
    {#if filteredHistory.length === 0}
      <div class="empty-state">
        <p class="text-muted text-center mb-0">No history</p>
      </div>
    {:else}
      {#each filteredHistory as entry (entry.id)}
        <div class="history-item">
          <div class="history-header">
            <div class="history-time">
              {new Date(entry.executedAt).toLocaleTimeString()}
            </div>
            <div class="history-db">
              {entry.databaseName}
            </div>
            <div class="history-time-ms">
              {entry.executionTime}ms
            </div>
            <div class="history-actions">
              <button
                class="btn btn-sm btn-link p-0"
                on:click={() => handleLoadQuery(entry.query)}
                title="Load to editor"
              >
                <i class="fas fa-arrow-left"></i>
              </button>
              <button
                class="btn btn-sm btn-link p-0 text-danger"
                on:click={() => handleDelete(entry.id)}
                title="Delete"
              >
                <i class="fas fa-trash"></i>
              </button>
            </div>
          </div>
          <div class="history-query">
            {entry.query.substring(0, 80)}...
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .query-history-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
  }

  .panel-header {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-tertiary);
  }

  .panel-header h6 {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .panel-toolbar {
    display: flex;
    gap: 4px;
    padding: 8px;
    border-bottom: 1px solid var(--border-color);
  }

  .panel-toolbar input {
    flex: 1;
  }

  .history-list {
    flex: 1;
    overflow-y: auto;
  }

  .empty-state {
    padding: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
  }

  .history-item {
    padding: 8px;
    border-bottom: 1px solid var(--border-light);
    transition: background-color 0.15s ease;
    cursor: pointer;
  }

  .history-item:hover {
    background-color: var(--hover-bg);
  }

  .history-header {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
  }

  .history-time {
    font-weight: 500;
    color: var(--text-primary);
    min-width: 50px;
  }

  .history-db {
    flex: 0 0 auto;
    color: var(--accent-blue);
    font-weight: 500;
  }

  .history-time-ms {
    flex: 0 0 auto;
    color: var(--text-muted);
    min-width: 40px;
  }

  .history-actions {
    display: flex;
    gap: 2px;
    margin-left: auto;
  }

  .history-actions button {
    color: var(--text-muted);
    font-size: 10px;
  }

  .history-actions button:hover {
    color: var(--text-primary);
  }

  .history-query {
    font-size: 11px;
    color: var(--text-secondary);
    margin-top: 4px;
    padding: 4px;
    background: var(--bg-primary);
    border-radius: 2px;
    max-height: 40px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
