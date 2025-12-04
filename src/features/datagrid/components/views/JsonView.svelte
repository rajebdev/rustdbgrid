<script>
  import { stringifyRowWithOrder } from "../../../../utils/dataFormatters";

  export let displayRows = [];
  export let displayData = null;
  export let isLoadingMore = false;
  export let hasMoreData = true;
  export let onLoadMore = null;
  export let onScroll = null;

  let containerElement;
  let lastScrollTop = 0;
  let lastLoadTriggeredAt = 0;

  function handleScroll() {
    if (!containerElement) return;

    const { scrollTop, scrollHeight, clientHeight } = containerElement;
    const distanceFromBottom = scrollHeight - (scrollTop + clientHeight);
    const scrolledToBottom = distanceFromBottom < 200;

    const now = Date.now();
    const isScrollingDown = scrollTop > lastScrollTop;
    const canTriggerLoad = now - lastLoadTriggeredAt > 1000;

    if (
      scrolledToBottom &&
      hasMoreData &&
      !isLoadingMore &&
      isScrollingDown &&
      canTriggerLoad
    ) {
      lastLoadTriggeredAt = now;
      if (onLoadMore) onLoadMore();
    }

    if (Math.abs(scrollTop - lastScrollTop) >= 5) {
      lastScrollTop = scrollTop;
      if (onScroll) onScroll({ scrollTop });
    }
  }
</script>

<div
  class="json-container flex-grow-1 p-3"
  bind:this={containerElement}
  on:scroll={handleScroll}
>
  <div class="json-list">
    {#each displayRows as row, index}
      <div class="json-item">
        <div class="json-item-header">
          <span class="json-item-number">#{index + 1}</span>
        </div>
        <pre class="json-content">{stringifyRowWithOrder(
            row,
            displayData.columns
          )}</pre>
      </div>
    {/each}
  </div>

  {#if isLoadingMore}
    <div class="text-center py-3 bg-light">
      <i class="fas fa-spinner fa-spin text-primary"></i>
      <span class="ms-2 text-muted">Loading more data...</span>
    </div>
  {/if}

  {#if !hasMoreData && displayRows.length > 0}
    <div class="text-center py-3 text-muted small bg-light">
      <i class="fas fa-check-circle"></i>
      <span class="ms-2"
        >All data loaded ({displayRows.length.toLocaleString()} rows)</span
      >
    </div>
  {/if}
</div>

<style>
  .json-container {
    position: relative;
    overflow: auto;
    height: 100%;
    background-color: var(--bg-tertiary);
  }

  .json-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .json-item {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 0.375rem;
    overflow: hidden;
    box-shadow: var(--shadow-sm);
  }

  .json-item-header {
    background: var(--accent-gradient);
    color: white;
    padding: 0.5rem 1rem;
    font-weight: 600;
    font-size: 0.875rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .json-item-number {
    background: var(--accent-number-bg);
    padding: 0.125rem 0.5rem;
    border-radius: 0.25rem;
    font-family: monospace;
  }

  .json-content {
    margin: 0;
    padding: 1rem;
    background-color: var(--bg-secondary);
    font-family: "Consolas", "Monaco", "Courier New", monospace;
    font-size: 0.875rem;
    line-height: 1.5;
    overflow-x: auto;
    color: var(--text-primary);
    border: none;
  }

  .json-content::-webkit-scrollbar {
    height: 8px;
  }

  .json-content::-webkit-scrollbar-track {
    background: var(--scrollbar-track);
  }

  .json-content::-webkit-scrollbar-thumb {
    background: var(--scrollbar-thumb);
    border-radius: 4px;
  }

  .json-content::-webkit-scrollbar-thumb:hover {
    background: var(--scrollbar-thumb-hover);
  }
</style>
