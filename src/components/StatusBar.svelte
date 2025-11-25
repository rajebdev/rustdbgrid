<script>
  import {
    activeConnection,
    selectedDatabase,
    selectedTable,
  } from "../stores/connections";
  import { tabDataStore } from "../stores/tabData";

  export let activeTabId = null;

  $: currentTabData = activeTabId ? $tabDataStore[activeTabId] : null;
</script>

<div
  class="d-flex justify-content-between align-items-center bg-light border-top px-2 py-1"
  style="height: 28px; font-size: 11px;"
>
  <div class="d-flex align-items-center gap-2">
    <div class="d-flex align-items-center gap-1">
      {#if $activeConnection}
        <span
          class="badge bg-success rounded-circle p-1"
          style="width: 8px; height: 8px;"
        ></span>
        <div class="d-flex align-items-center gap-1">
          <strong class="text-dark">{$activeConnection.name}</strong>
          <span class="text-muted">·</span>
          <span class="text-secondary">{$activeConnection.db_type}</span>
          <span class="text-muted">·</span>
          <span class="text-secondary"
            >{$activeConnection.host}:{$activeConnection.port}</span
          >
        </div>
      {:else}
        <span
          class="badge bg-secondary rounded-circle p-1"
          style="width: 8px; height: 8px;"
        ></span>
        <span class="text-muted">Not connected</span>
      {/if}
    </div>

    {#if $selectedDatabase}
      <span class="vr"></span>
      <div class="d-flex align-items-center gap-1">
        <i class="fas fa-database text-secondary"></i>
        <span class="text-dark">{$selectedDatabase.name}</span>
      </div>
    {/if}

    {#if $selectedTable}
      <span class="vr"></span>
      <div class="d-flex align-items-center gap-1">
        <i class="fas fa-table text-secondary"></i>
        <span class="text-dark">{$selectedTable.name}</span>
      </div>
    {/if}
  </div>

  <div class="d-flex align-items-center gap-2">
    {#if currentTabData?.queryResult}
      <div class="d-flex align-items-center gap-1">
        <i class="fas fa-list text-secondary"></i>
        <span class="text-dark">
          {currentTabData.queryResult.rows.length} rows
        </span>
      </div>
      <span class="vr"></span>
      <div class="d-flex align-items-center gap-1">
        <i class="fas fa-columns text-secondary"></i>
        <span class="text-dark">
          {currentTabData.queryResult.columns.length} columns
        </span>
      </div>
    {/if}
  </div>
</div>
