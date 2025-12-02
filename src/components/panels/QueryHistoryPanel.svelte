<script>
  import { queryHistoryStore } from "../../stores/queryHistory";
  import BasePanel from "./base/BasePanel.svelte";
  import PanelSearchBar from "./base/PanelSearchBar.svelte";
  import PanelEmptyState from "./base/PanelEmptyState.svelte";
  import { useSearch } from "../../composables/useSearch";
  import {
    formatTime,
    formatExecutionTime,
    truncateText,
    showConfirmation,
    dispatchLoadQuery,
  } from "../../utils/panelHelpers";

  const { searchTerm, filteredItems: filteredHistory } = useSearch(
    queryHistoryStore,
    ["query", "databaseName"]
  );

  function handleDelete(id) {
    if (showConfirmation("Delete this history entry?")) {
      queryHistoryStore.deleteEntry(id);
    }
  }

  function handleClearHistory() {
    if (showConfirmation("Clear all history?")) {
      queryHistoryStore.clearHistory();
    }
  }
</script>

<BasePanel title="Query History" icon="fas fa-history">
  <div slot="toolbar">
    <PanelSearchBar
      bind:value={$searchTerm}
      placeholder="Search history..."
      actionButton={{
        icon: "fas fa-times",
        variant: "outline-danger",
        title: "Clear history",
        onClick: handleClearHistory,
      }}
    />
  </div>

  {#if $filteredHistory.length === 0}
    <PanelEmptyState
      message={$queryHistoryStore.length === 0
        ? "No history"
        : "No matching history"}
      icon="fa-history"
      isSearchResult={$queryHistoryStore.length > 0}
    />
  {:else}
    {#each $filteredHistory as entry (entry.id)}
      <div class="panel-item">
        <div class="item-header">
          <div class="history-time">
            {formatTime(entry.executedAt)}
          </div>
          <div class="history-db">
            {entry.databaseName}
          </div>
          <div class="history-time-ms">
            {formatExecutionTime(entry.executionTime)}
          </div>
          <div class="item-actions">
            <button
              on:click={() => dispatchLoadQuery(entry.query)}
              title="Load to editor"
            >
              <i class="fas fa-arrow-left"></i>
            </button>
            <button
              class="text-danger"
              on:click={() => handleDelete(entry.id)}
              title="Delete"
            >
              <i class="fas fa-trash"></i>
            </button>
          </div>
        </div>
        <div class="item-content">
          {truncateText(entry.query)}
        </div>
      </div>
    {/each}
  {/if}
</BasePanel>

<style>
  .history-time {
    font-weight: 500;
    color: var(--text-primary);
    min-width: 50px;
    font-size: 11px;
  }

  .history-db {
    flex: 0 0 auto;
    color: var(--accent-blue);
    font-weight: 500;
    font-size: 11px;
  }

  .history-time-ms {
    flex: 0 0 auto;
    color: var(--text-muted);
    min-width: 40px;
    font-size: 11px;
  }
</style>
