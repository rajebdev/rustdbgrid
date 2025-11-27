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
      class="d-flex flex-column align-items-center justify-content-center h-100 text-secondary gap-3 bg-body-tertiary"
    >
      <i class="fas fa-spinner fa-spin" style="font-size: 48px; color: #0d6efd;"
      ></i>
      <p class="fs-6 m-0 text-dark">Loading table data...</p>
    </div>
  {/if}
</div>
