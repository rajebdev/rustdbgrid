<script>
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  export let tabs = [];
  export let activeTab = null;

  function selectTab(tab) {
    dispatch("select", tab);
  }

  function closeTab(event, tab) {
    event.stopPropagation();
    dispatch("close", tab);
  }

  function getTabIcon(type) {
    switch (type) {
      case "query":
        return "fa-file-code";
      case "table":
        return "fa-table";
      case "connection":
        return "fa-database";
      default:
        return "fa-file";
    }
  }
</script>

<div
  class="d-flex bg-body-secondary border-bottom"
  style="height: 32px; overflow: hidden; user-select: none;"
>
  {#if tabs.length === 0}
    <div
      class="d-flex align-items-center px-3 text-muted"
      style="font-size: 12px;"
    >
      <span>No tabs open</span>
    </div>
  {:else}
    <div
      class="d-flex flex-grow-1 overflow-auto align-items-end"
      style="scrollbar-width: thin;"
    >
      {#each tabs as tab (tab.id)}
        <div
          class="d-flex align-items-center gap-2 px-3 border-end position-relative {activeTab?.id ===
          tab.id
            ? 'bg-body border-top border-2 border-primary'
            : 'bg-body-tertiary'}"
          style="min-width: 120px; max-width: 200px; height: {activeTab?.id ===
          tab.id
            ? '32px'
            : '28px'}; transition: all 0.15s; {activeTab?.id === tab.id
            ? 'font-weight: 500;'
            : ''}"
        >
          <button
            class="d-flex align-items-center gap-2 flex-grow-1 border-0 bg-transparent p-0 text-start"
            style="cursor: pointer;"
            on:click={() => selectTab(tab)}
            title={tab.title}
          >
            <i
              class="fas {getTabIcon(tab.type)} text-secondary"
              style="font-size: 12px;"
            ></i>
            <span class="flex-grow-1 text-truncate" style="font-size: 12px;">
              {tab.title}
            </span>
            {#if tab.modified}
              <span class="text-primary" style="font-size: 14px;">●</span>
            {/if}
          </button>
          <button
            class="btn btn-sm p-0 border-0 bg-transparent text-secondary"
            style="width: 18px; height: 18px; font-size: 11px;"
            on:click={(e) => closeTab(e, tab)}
            title="Close"
          >
            <i class="fas fa-times"></i>
          </button>
        </div>
      {/each}
    </div>

    <div class="d-flex align-items-center px-1 border-start">
      <button
        class="btn btn-sm p-1 border-0 bg-transparent text-secondary"
        style="width: 28px; height: 24px; font-size: 12px;"
        title="New tab"
        on:click={() => dispatch("new")}
      >
        <i class="fas fa-plus"></i>
      </button>
    </div>
  {/if}
</div>

<style>
  /* Hover effect for non-active tabs */
  .bg-body-tertiary:hover {
    background-color: #e9ecef !important;
  }

  /* Hover effect for close button */
  .btn:hover {
    background-color: #dee2e6 !important;
  }
</style>
