<script>
  import { createEventDispatcher, onMount } from "svelte";
  import { tabStore } from "../../stores/tabs";
  import {
    connections,
    activeConnection,
    selectedDatabase,
  } from "../../stores/connections";
  import { getDatabases, getConnectionForEdit } from "../../utils/tauri";

  const dispatch = createEventDispatcher();
  const { activeTab } = tabStore;

  let databases = [];
  let loadingDatabases = false;

  function handleAction(action) {
    dispatch(action);
  }

  // Validate connection when connections list changes
  $: if ($connections.length > 0 && $activeConnection) {
    const connExists = $connections.find((c) => c.id === $activeConnection.id);
    if (!connExists) {
      // Active connection was deleted, clear it
      activeConnection.set(null);
      selectedDatabase.set(null);
    }
  }

  // Load databases when connection changes
  $: if ($activeConnection) {
    loadDatabasesForConnection($activeConnection);
  }

  async function loadDatabasesForConnection(conn) {
    if (!conn) {
      databases = [];
      return;
    }
    loadingDatabases = true;
    try {
      // Check if connection has ssl field (full config) or need to fetch it
      let fullConnection = conn;
      if (!conn.hasOwnProperty("ssl")) {
        fullConnection = await getConnectionForEdit(conn.id);
      }

      databases = await getDatabases(fullConnection);

      // Validate selected database still exists
      if (
        $selectedDatabase &&
        !databases.find((db) => db.name === $selectedDatabase)
      ) {
        selectedDatabase.set(null);
      }
    } catch (error) {
      console.error("Failed to load databases:", error);
      databases = [];
    }
    loadingDatabases = false;
  }

  function handleConnectionChange(event) {
    const connId = event.target.value;
    const conn = $connections.find((c) => c.id === connId);
    if (conn) {
      activeConnection.set(conn);
      selectedDatabase.set(null);
    }
  }

  function handleDatabaseChange(event) {
    selectedDatabase.set(event.target.value);
  }

  // Get connection and database info from active tab
  $: activeConnectionInfo = getConnectionInfo($activeTab);
  $: activeDatabase = getDatabaseInfo($activeTab);
  $: connectionDbType = getConnectionDbType($activeTab);
  $: databaseIcon = getDatabaseIcon($activeTab);
  $: isQueryTab = $activeTab?.type === "query";
  $: showConnectionSelector = isQueryTab;
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
      (connection?.db_type === "PostgreSQL" || connection?.db_type === "MSSQL")
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

  <div class="toolbar-divider"></div>

  <!-- Connection Selector and Database Selector - Positioned to the right -->
  {#if showConnectionSelector}
    <div style="margin-left: auto;" class="toolbar-right-section">
      <!-- Connection Selector -->
      <div class="toolbar-selector">
        <select
          class="toolbar-select"
          on:change={handleConnectionChange}
          value={$activeConnection?.id || ""}
        >
          <option value="" disabled>🔌 Connection</option>
          {#each $connections as conn}
            <option value={conn.id}
              >{#if conn.db_type === "MySQL"}🐬{:else if conn.db_type === "PostgreSQL"}🐘{:else if conn.db_type === "MongoDB"}🍃{:else if conn.db_type === "Redis"}📕{:else if conn.db_type === "Ignite"}🔥{:else if conn.db_type === "MSSQL"}🗄️{:else}🔌{/if}
              {conn.name}</option
            >
          {/each}
        </select>
      </div>

      <!-- Database Selector -->
      <div class="toolbar-selector">
        {#if loadingDatabases}
          <span class="toolbar-loading">
            <i class="fas fa-spinner fa-spin"></i>
          </span>
        {:else}
          <select
            class="toolbar-select"
            on:change={handleDatabaseChange}
            value={$selectedDatabase || ""}
            disabled={!$activeConnection || databases.length === 0}
          >
            <option value="" disabled>🛢️ Database</option>
            {#each databases as db}
              <option value={db.name}>🛢️ {db.name}</option>
            {/each}
          </select>
        {/if}
      </div>
    </div>
  {/if}

  <!-- Connection and Database Info - Only show for table/procedure tabs -->
  {#if showInfoPanel && (activeConnectionInfo || activeDatabase)}
    <div class="toolbar-divider"></div>
    <div class="toolbar-info">
      {#if activeConnectionInfo}
        <span class="info-item connection-item" title="Active Connection">
          {#if connectionDbType === "MySQL"}
            <span class="db-icon">🐬</span>
          {:else if connectionDbType === "PostgreSQL"}
            <span class="db-icon">🐘</span>
          {:else if connectionDbType === "MongoDB"}
            <span class="db-icon">🍃</span>
          {:else if connectionDbType === "Redis"}
            <i class="fas fa-database"></i>
          {:else if connectionDbType === "Ignite"}
            <span class="db-icon">🔥</span>
          {:else if connectionDbType === "MSSQL"}
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
    </div>
  {/if}
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 2px;
    background: var(--bg-toolbar);
    border-bottom: 1px solid var(--border-color);
    height: 28px;
    padding: 0 8px;
    user-select: none;
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
  }

  .toolbar-right-section {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .toolbar-selector {
    display: flex;
    align-items: center;
  }

  .toolbar-select {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 3px;
    height: 22px;
    min-width: 120px;
    max-width: 180px;
    cursor: pointer;
  }

  .toolbar-select:hover {
    border-color: var(--accent-blue);
  }

  .toolbar-select:focus {
    outline: none;
    border-color: var(--accent-blue);
  }

  .toolbar-select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .toolbar-loading {
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 120px;
    height: 22px;
    color: var(--text-muted);
    font-size: 11px;
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
