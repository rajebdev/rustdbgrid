<script>
  import DataGrid from "../../../datagrid/components/DataGrid.svelte";
  import { loadTableData } from "../../../../core/integrations/tauri";
  import { tabDataStore } from "../../../../shared/stores/tabData";

  export let tabId;
  export let currentTabData;
  export let tableInfo;
  export let connection;

  let isReconnecting = false;
  let isAutoLoading = false;

  $: connectionError = currentTabData?.error;

  // Auto-load data if missing
  $: if (
    currentTabData &&
    !currentTabData?.queryResult &&
    !connectionError &&
    !isAutoLoading &&
    connection &&
    tableInfo
  ) {
    autoLoadData();
  }

  async function autoLoadData() {
    isAutoLoading = true;

    try {
      const tableData = await loadTableData(
        connection.id,
        connection.db_type,
        tableInfo.name,
        {
          database: tableInfo.database,
          schema: tableInfo.schema || null,
          limit: 200,
          offset: 0,
          filters: [],
          orderBy: [],
        }
      );

      tabDataStore.setQueryResult(tabId, tableData);

      if (tableData.final_query) {
        tabDataStore.setExecutedQuery(tabId, tableData.final_query);
      }

      tabDataStore.clearError(tabId);
    } catch (error) {
      tabDataStore.setError(
        tabId,
        error.message || "Failed to load table data"
      );
    } finally {
      isAutoLoading = false;
    }
  }

  async function handleReconnect() {
    isReconnecting = true;
    tabDataStore.clearError(tabId);

    try {
      const tableData = await loadTableData(
        connection.id,
        connection.db_type,
        tableInfo.name,
        {
          database: tableInfo.database,
          schema: tableInfo.schema || null,
          limit: 200,
          offset: 0,
          filters: [],
          orderBy: [],
        }
      );

      // Update the tab data with new results
      tabDataStore.setQueryResult(tabId, tableData);

      if (tableData.final_query) {
        tabDataStore.setExecutedQuery(tabId, tableData.final_query);
      }
    } catch (error) {
      tabDataStore.setError(
        tabId,
        error.message || "Failed to reconnect to database"
      );
    } finally {
      isReconnecting = false;
    }
  }
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
  {:else if connectionError}
    <div class="error-container">
      <i class="fas fa-exclamation-circle"></i>
      <p class="error-message">{connectionError}</p>
      <p class="error-subtitle">Unable to connect to database</p>
      <button
        class="reconnect-btn"
        on:click={handleReconnect}
        disabled={isReconnecting}
      >
        {#if isReconnecting}
          <i class="fas fa-spinner fa-spin"></i>
          <span>Reconnecting...</span>
        {:else}
          <i class="fas fa-sync-alt"></i>
          <span>Reconnect Database</span>
        {/if}
      </button>
    </div>
  {:else}
    <div class="loading-container">
      <i class="fas fa-spinner fa-spin"></i>
      <p>{isAutoLoading ? "Loading table data..." : "Loading..."}</p>
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

  .error-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 16px;
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    padding: 24px;
  }

  .error-container i {
    font-size: 64px;
    color: var(--accent-red, #ff6b6b);
  }

  .error-message {
    margin: 0;
    font-size: 16px;
    font-weight: 500;
    color: var(--text-primary);
    text-align: center;
    max-width: 400px;
  }

  .error-subtitle {
    margin: 0;
    font-size: 13px;
    color: var(--text-secondary);
    text-align: center;
  }

  .reconnect-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 20px;
    background: var(--accent-blue);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    transition: all 0.2s ease;
    margin-top: 8px;
  }

  .reconnect-btn:hover:not(:disabled) {
    background: var(--accent-blue-hover, #4a9eff);
    transform: translateY(-1px);
  }

  .reconnect-btn:active:not(:disabled) {
    transform: translateY(0);
  }

  .reconnect-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .reconnect-btn i {
    font-size: 14px;
  }
</style>
