<script>
  import DataGrid from "../common/DataGrid.svelte";
  import { activeConnection } from "../../stores/connections";

  export let tabId;
  export let currentTabData;
  export let tableInfo;
</script>

<div class="flex-grow-1 overflow-hidden">
  {#if currentTabData?.queryResult}
    <DataGrid
      data={currentTabData.queryResult}
      {tabId}
      executedQuery={currentTabData?.executedQuery || ""}
      connection={tableInfo?.connection || $activeConnection}
      tableName={tableInfo?.name || ""}
      databaseName={tableInfo?.database || ""}
    />
  {:else}
    <div
      class="loading-container d-flex flex-column align-items-center justify-content-center h-100 gap-3"
    >
      <i class="fas fa-spinner fa-spin loading-spinner"></i>
      <p class="fs-6 m-0 loading-text">Loading table data...</p>
    </div>
  {/if}
</div>

<style>
  .loading-container {
    background: var(--bg-tertiary);
  }

  .loading-spinner {
    font-size: 48px;
    color: var(--accent-blue);
  }

  .loading-text {
    color: var(--text-primary);
  }
</style>
