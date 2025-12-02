<script>
  import DataGrid from "../../common/DataGrid.svelte";

  export let tabId;
  export let currentTabData;
  export let tableInfo;
  export let connection;
</script>

<div class="data-tab-container">
  {#if currentTabData?.queryResult}
    <DataGrid
      data={currentTabData.queryResult}
      {tabId}
      executedQuery={currentTabData?.executedQuery || ""}
      {connection}
      tableName={tableInfo?.name || ""}
      databaseName={tableInfo?.database || ""}
      schemaName={tableInfo?.schema || ""}
    />
  {:else}
    <div class="loading-container">
      <i class="fas fa-spinner fa-spin"></i>
      <p>Loading table data...</p>
    </div>
  {/if}
</div>

<style>
  .data-tab-container {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .loading-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 12px;
    background: var(--bg-tertiary);
    color: var(--text-secondary);
  }

  .loading-container i {
    font-size: 48px;
    color: var(--accent-blue);
  }

  .loading-container p {
    margin: 0;
    font-size: 14px;
  }
</style>
