<script>
  import { onMount, onDestroy } from "svelte";
  import SqlEditor from "../common/SqlEditor.svelte";
  import DataGrid from "../common/DataGrid.svelte";
  import { activeConnection } from "../../stores/connections";
  import { tabDataStore } from "../../stores/tabData";

  export let tabId;
  export let tab;
  export let currentTabData;
  export let editorHeight = 300;
  export let isResizingEditor = false;

  // Internal result tabs
  let resultTabs = [];
  let activeResultTabId = null;
  let hasUserClosedAllTabs = false;
  let isExecuting = false;

  // Initialize result tabs from currentTabData
  $: if (
    currentTabData?.queryResult &&
    resultTabs.length === 0 &&
    !hasUserClosedAllTabs
  ) {
    // Initial result tab
    resultTabs = [
      {
        id: 1,
        title: "Result 1",
        result: currentTabData.queryResult,
        query: currentTabData.executedQuery || "",
        executedAt: new Date().toISOString(),
      },
    ];
    activeResultTabId = 1;
  }

  // Listen for query execution start
  function handleQueryExecutionStart(event) {
    if (event.detail.tabId !== tabId) return;
    isExecuting = true;
  }

  // Listen for query execution end
  function handleQueryExecutionEnd(event) {
    if (event.detail.tabId !== tabId) return;
    isExecuting = false;
  }

  // Listen for new tab result events
  function handleExecuteNewTab(event) {
    if (event.detail.tabId !== tabId) return;

    hasUserClosedAllTabs = false;
    const newTabId = Date.now();
    const newTab = {
      id: newTabId,
      title: `Result ${resultTabs.length + 1}`,
      result: event.detail.result,
      query: event.detail.query,
      executedAt: new Date().toISOString(),
    };

    resultTabs = [...resultTabs, newTab];
    activeResultTabId = newTabId;
  }

  // Update current tab result (when executing normally)
  function handleUpdateResult(event) {
    if (event.detail.tabId !== tabId) return;

    hasUserClosedAllTabs = false;

    if (resultTabs.length === 0) {
      // Create first tab
      resultTabs = [
        {
          id: 1,
          title: "Result 1",
          result: event.detail.result,
          query: event.detail.query,
          executedAt: new Date().toISOString(),
        },
      ];
      activeResultTabId = 1;
    } else {
      // Update active tab
      resultTabs = resultTabs.map((tab) =>
        tab.id === activeResultTabId
          ? {
              ...tab,
              result: event.detail.result,
              query: event.detail.query,
              executedAt: new Date().toISOString(),
            }
          : tab
      );
    }
  }

  function selectResultTab(tabIdToSelect) {
    activeResultTabId = tabIdToSelect;
  }

  function closeResultTab(tabIdToClose) {
    console.log("closeResultTab called with tabId:", tabIdToClose);
    console.log("Current resultTabs:", resultTabs);
    console.log("activeResultTabId:", activeResultTabId);

    const index = resultTabs.findIndex((t) => t.id === tabIdToClose);
    resultTabs = resultTabs.filter((t) => t.id !== tabIdToClose);

    console.log("After filter resultTabs:", resultTabs);
    console.log("resultTabs.length:", resultTabs.length);

    if (activeResultTabId === tabIdToClose) {
      if (resultTabs.length > 0) {
        // Select next tab or previous if closing active tab
        const newIndex = Math.min(index, resultTabs.length - 1);
        activeResultTabId = resultTabs[newIndex]?.id || null;
      } else {
        // No tabs left, hide result view
        console.log("Setting activeResultTabId to null");
        activeResultTabId = null;
        hasUserClosedAllTabs = true;
      }
    }

    // Force reactivity
    resultTabs = resultTabs;
    console.log(
      "Final state - resultTabs.length:",
      resultTabs.length,
      "activeResultTabId:",
      activeResultTabId
    );
  }

  $: activeResultTab = resultTabs.find((t) => t.id === activeResultTabId);

  onMount(() => {
    window.addEventListener("query-execution-start", handleQueryExecutionStart);
    window.addEventListener("query-execution-end", handleQueryExecutionEnd);
    window.addEventListener("execute-new-result-tab", handleExecuteNewTab);
    window.addEventListener("update-result-tab", handleUpdateResult);

    return () => {
      window.removeEventListener(
        "query-execution-start",
        handleQueryExecutionStart
      );
      window.removeEventListener(
        "query-execution-end",
        handleQueryExecutionEnd
      );
      window.removeEventListener("execute-new-result-tab", handleExecuteNewTab);
      window.removeEventListener("update-result-tab", handleUpdateResult);
    };
  });

  function handleEditorMouseDown(event) {
    event.preventDefault();
    window.dispatchEvent(
      new CustomEvent("start-editor-resize", { detail: { event } })
    );
  }

  function handleEditorMouseMove(event) {
    if (isResizingEditor) {
      window.dispatchEvent(
        new CustomEvent("editor-resize-move", { detail: { event } })
      );
    }
  }
</script>

<div
  class="d-flex flex-column h-100"
  class:resizing={isResizingEditor}
  on:mousemove={handleEditorMouseMove}
  role="presentation"
>
  <div
    class="border-bottom-2 border-secondary position-relative"
    style="height: {resultTabs.length > 0
      ? `${editorHeight}px`
      : '100%'}; flex-shrink: 0; overflow: hidden;"
  >
    {#key tabId}
      <SqlEditor {tabId} {tab} />
    {/key}
  </div>

  {#if resultTabs.length > 0}
    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      class="editor-resize-handle border-top border-bottom"
      style="height: 6px; cursor: row-resize;"
      on:mousedown={handleEditorMouseDown}
      role="separator"
      aria-orientation="horizontal"
      aria-label="Resize editor"
    ></div>

    <!-- Result Tabs Bar -->
    <div class="result-tabs-bar d-flex align-items-center border-bottom">
      {#each resultTabs as tab (tab.id)}
        <div
          class="result-tab d-flex align-items-center gap-1 px-2 py-1"
          class:active={tab.id === activeResultTabId}
          on:click={() => selectResultTab(tab.id)}
          on:keydown={(e) => e.key === "Enter" && selectResultTab(tab.id)}
          role="tab"
          tabindex="0"
          aria-selected={tab.id === activeResultTabId}
          title={tab.query?.substring(0, 100) || tab.title}
        >
          <i class="fas fa-table" style="font-size: 10px;"></i>
          <span class="tab-title">{tab.title}</span>
          <button
            class="btn-close-tab"
            on:click|stopPropagation={() => closeResultTab(tab.id)}
            title="Close tab"
          >
            <i class="fas fa-times"></i>
          </button>
        </div>
      {/each}
    </div>

    <!-- Active Result Content -->
    <div class="flex-grow-1 overflow-hidden position-relative">
      {#if activeResultTab}
        {#key activeResultTabId}
          <DataGrid
            data={activeResultTab.result}
            tabId={`${tabId}-${activeResultTabId}`}
            executedQuery={activeResultTab.query || ""}
            connection={$activeConnection}
          />
        {/key}
      {/if}

      {#if isExecuting}
        <div class="loading-overlay">
          <div class="loading-content">
            <i class="fas fa-spinner fa-spin"></i>
            <p>Executing query...</p>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .editor-resize-handle {
    display: block;
    background: var(--bg-tertiary);
    transition: background-color 0.2s;
  }

  .editor-resize-handle:hover {
    background-color: var(--accent-blue) !important;
  }

  .editor-resize-handle:focus {
    outline: 2px solid var(--accent-blue);
    outline-offset: -2px;
  }

  .resizing {
    cursor: row-resize !important;
  }

  .resizing :global(*) {
    user-select: none !important;
    pointer-events: none !important;
    cursor: row-resize !important;
  }

  .result-tabs-bar {
    background: var(--bg-tertiary);
    min-height: 28px;
    overflow-x: auto;
    gap: 1px;
  }

  .result-tab {
    background: var(--bg-secondary);
    cursor: pointer;
    font-size: 11px;
    color: var(--text-secondary);
    border-right: 1px solid var(--border-color);
    white-space: nowrap;
    transition: all 0.15s ease;
  }

  .result-tab:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .result-tab.active {
    background: var(--bg-primary);
    color: var(--text-primary);
    border-bottom: 2px solid var(--accent-blue);
  }

  .tab-title {
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .btn-close-tab {
    background: none;
    border: none;
    color: var(--text-muted);
    padding: 0 2px;
    font-size: 9px;
    cursor: pointer;
    opacity: 0.6;
    transition: opacity 0.15s ease;
  }

  .btn-close-tab:hover {
    opacity: 1;
    color: var(--danger);
  }

  .loading-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .loading-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    background: var(--bg-secondary);
    padding: 24px 40px;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .loading-content i {
    font-size: 48px;
    color: var(--accent-blue);
  }

  .loading-content p {
    margin: 0;
    font-size: 14px;
    color: var(--text-primary);
    font-weight: 500;
  }
</style>
