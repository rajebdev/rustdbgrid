<script>
  import { activeConnection } from "../../../connection/stores/connections";
  import { tabDataStore } from "../../../../shared/stores/tabData";
  import { DatabaseType } from "../../../../core/config/databaseTypes";
  import { getTabsForDatabase } from "../../../../shared/utils/config/propertiesTabConfig";
  import { loadAllTableData } from "../../services/propertiesDataService";

  // Tab content components
  import ColumnsTabContent from "./properties/ColumnsTabContent.svelte";
  import ConstraintsTabContent from "./properties/ConstraintsTabContent.svelte";
  import ForeignKeysTabContent from "./properties/ForeignKeysTabContent.svelte";
  import IndexesTabContent from "./properties/IndexesTabContent.svelte";
  import ReferencesTabContent from "./properties/ReferencesTabContent.svelte";
  import TriggersTabContent from "./properties/TriggersTabContent.svelte";
  import PartitionsTabContent from "./properties/PartitionsTabContent.svelte";
  import StatisticsTabContent from "./properties/StatisticsTabContent.svelte";
  import DDLTabContent from "./properties/DDLTabContent.svelte";
  import VirtualTabContent from "./properties/VirtualTabContent.svelte";
  import NotImplementedTabContent from "./properties/NotImplementedTabContent.svelte";

  export let tabId;
  export let tableInfo;
  export let connection;

  // State
  let tableSchema = null;
  let tableStatistics = null;
  let tableReferences = null;
  let tableTriggers = null;
  let pgData = null;
  let loading = true;
  let error = null;

  // Get database type and connection
  $: conn = connection || $activeConnection;
  $: isMssql = conn?.db_type === DatabaseType.MSSQL;
  $: isPostgres = conn?.db_type === DatabaseType.POSTGRESQL;

  // Get tabs based on database type
  $: tabs = getTabsForDatabase(conn?.db_type);

  // Subscribe to tabDataStore to get reactive updates
  $: tabData = $tabDataStore[tabId] || {};
  $: activeTab = tabData.activePropertiesTab || "Columns";

  function handlePropertiesTabChange(newTab) {
    tabDataStore.setActivePropertiesTab(tabId, newTab);
  }

  // Reactive statement to reload data when tableInfo changes
  $: if (tableInfo && conn) {
    loadData();
  }

  async function loadData() {
    try {
      loading = true;
      error = null;

      const data = await loadAllTableData(conn, tableInfo);

      tableSchema = data.schema;
      tableStatistics = data.statistics;
      tableReferences = data.references;
      tableTriggers = data.triggers;
      pgData = data.pgData;
    } catch (e) {
      console.error("Error loading table data:", e);
      error = e.message || "Failed to load table data";
    } finally {
      loading = false;
    }
  }
</script>

<div class="properties-container">
  <!-- Table Info Header -->
  <div class="table-info-header">
    <div class="info-row">
      <span class="info-label">Table Name:</span>
      <span class="info-value">{tableInfo?.name || "N/A"}</span>
    </div>
    <div class="info-row">
      <span class="info-label">Engine:</span>
      <span class="info-value">{tableSchema?.engine || "InnoDB"}</span>
    </div>
    <div class="info-row">
      <span class="info-label">Collation:</span>
      <span class="info-value"
        >{tableSchema?.collation || "latin1_swedish_ci"}</span
      >
    </div>
    <div class="info-row">
      <label>
        <input
          type="checkbox"
          checked={tableSchema?.partitioned || false}
          readonly
          style="pointer-events: none;"
        />
        Partitioned
      </label>
    </div>
    <div class="info-row">
      <span class="info-label">Auto Increment:</span>
      <input
        type="text"
        class="info-input"
        value={tableSchema?.auto_increment || "11"}
      />
    </div>
  </div>

  <!-- Main content with sidebar -->
  <div class="properties-main">
    <!-- Left Sidebar with vertical tabs -->
    <div class="properties-sidebar">
      {#each tabs as tab}
        <button
          class="sidebar-tab-btn"
          class:active={activeTab === tab.id}
          on:click={() => handlePropertiesTabChange(tab.id)}
        >
          <i class={tab.icon}></i>
          <span>{tab.label}</span>
        </button>
      {/each}
    </div>

    <!-- Content Area -->
    <div class="properties-content">
      {#if loading}
        <div class="loading-state">
          <i class="fas fa-spinner fa-spin"></i>
          <span>Loading schema...</span>
        </div>
      {:else if error}
        <div class="error-state">
          <i class="fas fa-exclamation-triangle"></i>
          <span>{error}</span>
        </div>
      {:else if activeTab === "Columns"}
        <ColumnsTabContent {tableSchema} />
      {:else if activeTab === "Constraints"}
        <ConstraintsTabContent
          {tableSchema}
          pgConstraints={pgData?.constraints}
          {isPostgres}
          loadingPgData={loading}
        />
      {:else if activeTab === "Foreign Keys"}
        <ForeignKeysTabContent
          {tableSchema}
          pgForeignKeys={pgData?.foreignKeys}
          {tableInfo}
          {isPostgres}
          loadingPgData={loading}
        />
      {:else if activeTab === "Indexes"}
        <IndexesTabContent
          {tableSchema}
          pgIndexes={pgData?.indexes}
          {tableInfo}
          {isPostgres}
          loadingPgData={loading}
        />
      {:else if activeTab === "References"}
        <ReferencesTabContent
          {tableReferences}
          pgReferences={pgData?.references}
          {isPostgres}
          loadingReferences={loading}
          loadingPgData={loading}
        />
      {:else if activeTab === "Triggers"}
        <TriggersTabContent {tableTriggers} loadingTriggers={loading} />
      {:else if activeTab === "Partitions"}
        <PartitionsTabContent
          pgPartitions={pgData?.partitions}
          {tableSchema}
          {isPostgres}
          loadingPgData={loading}
        />
      {:else if activeTab === "Statistics"}
        <StatisticsTabContent
          {tableStatistics}
          {tableSchema}
          {isMssql}
          {isPostgres}
        />
      {:else if activeTab === "DDL"}
        <DDLTabContent {tableInfo} {tableSchema} />
      {:else if activeTab === "Virtual"}
        <VirtualTabContent {tableSchema} />
      {:else}
        <NotImplementedTabContent {activeTab} />
      {/if}
    </div>
  </div>

  <!-- Footer with item count -->
  <div class="properties-footer">
    <div class="footer-left">
      <button class="footer-btn" title="Refresh">
        <i class="fas fa-sync-alt"></i>
        <span>Refresh</span>
      </button>
      <button class="footer-btn" title="Save">
        <i class="fas fa-save"></i>
        <span>Save</span>
      </button>
      <button class="footer-btn" title="Revert">
        <i class="fas fa-undo"></i>
        <span>Revert</span>
      </button>
      <button class="footer-btn" title="Add Column">
        <i class="fas fa-plus"></i>
        <span>Add</span>
      </button>
      <button class="footer-btn" title="Delete">
        <i class="fas fa-trash"></i>
        <span>Delete</span>
      </button>
    </div>
    <div class="footer-right">
      <span>{tableSchema?.columns?.length || 0} items</span>
    </div>
  </div>
</div>

<style>
  .properties-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
  }

  .table-info-header {
    display: flex;
    gap: 16px;
    padding: 8px 12px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    font-size: 12px;
  }

  .info-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .info-label {
    color: var(--text-secondary);
    font-weight: 500;
  }

  .info-value {
    color: var(--text-primary);
  }

  .info-input {
    width: 60px;
    padding: 2px 6px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 3px;
    color: var(--text-primary);
    font-size: 12px;
  }

  .properties-main {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .properties-sidebar {
    display: flex;
    flex-direction: column;
    width: 160px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
    overflow-y: auto;
  }

  .sidebar-tab-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: transparent;
    border: none;
    border-left: 3px solid transparent;
    color: var(--text-secondary);
    font-size: 12px;
    text-align: left;
    cursor: pointer;
    white-space: nowrap;
    transition: all 0.2s ease;
    height: 28px;
  }

  .sidebar-tab-btn i {
    font-size: 11px;
    width: 16px;
    text-align: center;
  }

  .sidebar-tab-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .sidebar-tab-btn.active {
    background: var(--bg-primary);
    color: var(--accent-blue);
    border-left-color: var(--accent-blue);
    font-weight: 500;
  }

  .properties-content {
    flex: 1;
    overflow: auto;
    background: var(--bg-primary);
  }

  .loading-state,
  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 12px;
    color: var(--text-secondary);
  }

  .loading-state i,
  .error-state i {
    font-size: 32px;
  }

  .error-state {
    color: var(--error-color);
  }

  .properties-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 2px 8px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
    font-size: 10px;
    height: 24px;
  }

  .footer-left {
    display: flex;
    gap: 1px;
  }

  .footer-btn {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 2px 8px;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 10px;
    cursor: pointer;
    transition: all 0.15s ease;
    border-radius: 3px;
    height: 100%;
  }

  .footer-btn i {
    font-size: 11px;
  }

  .footer-btn span {
    font-size: 10px;
    white-space: nowrap;
  }

  .footer-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .footer-btn:active {
    background: var(--bg-primary);
  }

  .footer-right {
    color: var(--text-secondary);
    padding-right: 4px;
    font-size: 10px;
  }
</style>
