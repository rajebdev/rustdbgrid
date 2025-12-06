<script>
  import { createEventDispatcher } from "svelte";
  import { tabStore } from "../../stores/tabs";
  import { DatabaseType } from "../../../core/config/databaseTypes";

  const dispatch = createEventDispatcher();
  const { activeTab } = tabStore;

  function handleAction(action) {
    dispatch(action);
  }

  // Get connection and database info from active tab
  $: activeConnectionInfo = getConnectionInfo($activeTab);
  $: activeDatabase = getDatabaseInfo($activeTab);
  $: connectionDbType = getConnectionDbType($activeTab);
  $: databaseIcon = getDatabaseIcon($activeTab);
  $: showInfoPanel =
    $activeTab?.type === "table" || $activeTab?.type === "procedure";

  function getConnectionInfo(tab) {
    if (!tab) return null;
    if (tab.type === "table" && tab.tableInfo?.connection) {
      return tab.tableInfo.connection.name;
    }
    if (tab.type === "procedure" && tab.procedureInfo?.connection) {
      return tab.procedureInfo.connection.name;
    }
    return null;
  }

  function getConnectionDbType(tab) {
    if (!tab) return null;
    if (tab.type === "table" && tab.tableInfo?.connection) {
      return tab.tableInfo.connection.db_type;
    }
    if (tab.type === "procedure" && tab.procedureInfo?.connection) {
      return tab.procedureInfo.connection.db_type;
    }
    return null;
  }

  function getDatabaseInfo(tab) {
    if (!tab) return null;

    let database = null;
    let schema = null;
    let connection = null;

    if (tab.type === "table" && tab.tableInfo) {
      database = tab.tableInfo.database;
      schema = tab.tableInfo.schema;
      connection = tab.tableInfo.connection;
    } else if (tab.type === "procedure" && tab.procedureInfo) {
      database = tab.procedureInfo.database;
      schema = tab.procedureInfo.schema;
      connection = tab.procedureInfo.connection;
    }

    if (!database) return null;

    // Format: schema@database for PostgreSQL/MSSQL with schema, otherwise just database
    if (
      schema &&
      (connection?.db_type === DatabaseType.POSTGRESQL ||
        connection?.db_type === DatabaseType.MSSQL)
    ) {
      return `${schema}@${database}`;
    }

    return database;
  }

  function getDatabaseIcon(tab) {
    if (!tab) return null;
    if (tab.type === "table" && tab.tableInfo?.connection) {
      return tab.tableInfo.connection.db_type;
    }
    if (tab.type === "procedure" && tab.procedureInfo?.connection) {
      return tab.procedureInfo.connection.db_type;
    }
    return null;
  }
</script>

<div class="toolbar">
  <button
    class="toolbar-btn"
    title="New Connection"
    on:click={() => handleAction("newConnection")}
  >
    <i class="fas fa-plus"></i>
  </button>
  <div class="toolbar-divider"></div>
  <button
    class="toolbar-btn"
    title="New SQL Script (Ctrl+N)"
    on:click={() => handleAction("newQuery")}
  >
    <i class="fas fa-file-code"></i>
  </button>
  <button
    class="toolbar-btn"
    title="Save (Ctrl+S)"
    on:click={() => handleAction("saveQuery")}
  >
    <i class="fas fa-save"></i>
  </button>
  <div class="toolbar-divider"></div>
  <button
    class="toolbar-btn"
    title="Execute SQL (F5 / Ctrl+Enter)"
    on:click={() => handleAction("execute")}
  >
    <i class="fas fa-play icon-success"></i>
  </button>
  <button
    class="toolbar-btn"
    title="Execute Script (Ctrl+Shift+Enter)"
    on:click={() => handleAction("executeScript")}
  >
    <i class="fas fa-play-circle icon-success"></i>
  </button>
  <button
    class="toolbar-btn"
    title="Stop"
    on:click={() => handleAction("stop")}
  >
    <i class="fas fa-stop icon-danger"></i>
  </button>
  <div class="toolbar-divider"></div>
  <button
    class="toolbar-btn"
    title="Refresh (Shift+F5)"
    on:click={() => handleAction("refresh")}
  >
    <i class="fas fa-sync-alt"></i>
  </button>

  <!-- Connection and Database Info - Always reserve space -->
  <div class="toolbar-info">
    {#if showInfoPanel && (activeConnectionInfo || activeDatabase)}
      <div class="toolbar-divider"></div>
      {#if activeConnectionInfo}
        <span class="info-item connection-item" title="Active Connection">
          {#if connectionDbType === DatabaseType.MYSQL}
            <span class="db-icon">🐬</span>
          {:else if connectionDbType === DatabaseType.POSTGRESQL}
            <span class="db-icon">🐘</span>
          {:else if connectionDbType === DatabaseType.MONGODB}
            <span class="db-icon">🍃</span>
          {:else if connectionDbType === DatabaseType.REDIS}
            <i class="fas fa-database"></i>
          {:else if connectionDbType === DatabaseType.IGNITE}
            <span class="db-icon">🔥</span>
          {:else if connectionDbType === DatabaseType.MSSQL}
            <span class="db-icon">🗄️</span>
          {:else}
            <i class="fas fa-server"></i>
          {/if}
          <span class="info-text">{activeConnectionInfo}</span>
        </span>
      {/if}
      {#if activeDatabase}
        <span class="info-item database-item" title="Active Database">
          <i class="fas fa-database"></i>
          <span class="info-text">{activeDatabase}</span>
        </span>
      {/if}
    {/if}
  </div>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 2px;
    background: var(--bg-toolbar);
    border-bottom: 1px solid var(--border-color);
    height: 28px;
    min-height: 28px;
    max-height: 28px;
    padding: 0 8px;
    user-select: none;
    overflow: hidden;
    flex-shrink: 0;
  }

  .toolbar-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    padding: 4px 6px;
    font-size: 13px;
    cursor: pointer;
    transition: all 0.15s;
    border-radius: 3px;
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    height: 22px;
  }

  .toolbar-btn:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .toolbar-btn:active {
    background: var(--border-color);
    transform: translateY(1px);
  }

  .toolbar-divider {
    width: 1px;
    height: 16px;
    background: var(--border-color);
    margin: 0 4px;
  }

  .icon-success {
    color: var(--accent-green);
  }

  .icon-danger {
    color: var(--accent-red);
  }

  .toolbar-info {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-left: auto;
    padding-right: 8px;
    min-height: 22px;
    max-height: 22px;
    overflow: hidden;
    /* Always reserve minimum space to prevent layout shift */
  }

  .info-item {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-secondary);
    font-size: 12px;
    white-space: nowrap;
  }

  .info-item i {
    color: var(--text-muted);
    font-size: 11px;
  }

  .connection-item i {
    color: var(--accent-blue, #3b82f6);
  }

  .database-item i {
    color: var(--accent-orange, #f59e0b);
  }

  .info-text {
    font-weight: 500;
  }

  .db-icon {
    font-size: 12px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
</style>
