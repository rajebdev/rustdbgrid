<script>
  import { onMount, createEventDispatcher } from "svelte";
  import {
    connections,
    activeConnection,
    selectedDatabase,
    selectedTable,
  } from "../stores/connections";
  import {
    getConnections,
    getDatabases,
    getTables,
    deleteConnection,
    connectToDatabase,
    disconnectFromDatabase,
    isDatabaseConnected,
    getConnectedDatabases,
  } from "../utils/tauri";

  const dispatch = createEventDispatcher();
  import ConnectionModal from "./ConnectionModal.svelte";
  import ConnectionContextMenu from "./ConnectionContextMenu.svelte";

  export let onToggleSidebar = null;

  let databases = [];
  let tables = [];
  let showModal = false;
  let editingConnection = null;
  let expandedConnections = {};
  let expandedDatabases = {};
  let expandedTables = {};
  let expandedSchemas = {};
  let expandedSchemasParent = {};
  let searchQuery = "";
  let loadingConnections = {}; // Track loading state per connection
  let loadingDatabases = {}; // Track loading state per database
  let connectedConnections = {}; // Track connection status
  let contextMenu = null; // { x, y, connection }

  onMount(async () => {
    await loadConnections();
    // Load connected databases from backend
    await syncConnectedStatus();
    // Close context menu when clicking anywhere
    document.addEventListener("click", closeContextMenu);
    return () => {
      document.removeEventListener("click", closeContextMenu);
    };
  });

  async function syncConnectedStatus() {
    try {
      const connectedIds = await getConnectedDatabases();
      const newConnectedConnections = {};
      for (const id of connectedIds) {
        newConnectedConnections[id] = true;
      }
      connectedConnections = newConnectedConnections;
    } catch (error) {
      console.error("Failed to sync connected status:", error);
    }
  }

  async function loadConnections() {
    try {
      const conns = await getConnections();
      connections.set(conns);
    } catch (error) {
      console.error("Failed to load connections:", error);
    }
  }

  async function toggleConnection(conn) {
    const isExpanded = expandedConnections[conn.id];

    if (!isExpanded) {
      activeConnection.set(conn);
      loadingConnections[conn.id] = true;
      loadingConnections = { ...loadingConnections };
      try {
        // Connect to database via backend
        await connectToDatabase(conn);
        databases = await getDatabases(conn);
        expandedConnections[conn.id] = { databases };
        connectedConnections[conn.id] = true; // Mark as connected
        connectedConnections = { ...connectedConnections };
      } catch (error) {
        console.error("Failed to load databases:", error);
      } finally {
        loadingConnections[conn.id] = false;
        loadingConnections = { ...loadingConnections };
      }
    } else {
      delete expandedConnections[conn.id];
    }
    expandedConnections = { ...expandedConnections };
  }

  async function toggleDatabase(connId, db) {
    const key = `${connId}-${db.name}`;
    const isExpanded = expandedDatabases[key];

    if (!isExpanded) {
      // Find and set the correct connection for this database
      const conn = $connections.find((c) => c.id === connId);
      if (conn) {
        activeConnection.set(conn);
      }

      selectedDatabase.set(db);
      loadingDatabases[key] = true;
      loadingDatabases = { ...loadingDatabases };
      try {
        const dbTables = await getTables($activeConnection, db.name);
        expandedDatabases[key] = {
          tables: dbTables,
          connection: conn,
          database: db,
        };
      } catch (error) {
        console.error("Failed to load tables:", error);
      } finally {
        loadingDatabases[key] = false;
        loadingDatabases = { ...loadingDatabases };
      }
    } else {
      delete expandedDatabases[key];
    }
    expandedDatabases = { ...expandedDatabases };
  }

  function toggleTables(connId, dbName) {
    const key = `${connId}-${dbName}`;
    if (expandedTables[key]) {
      delete expandedTables[key];
    } else {
      expandedTables[key] = true;
    }
    expandedTables = { ...expandedTables };
  }

  function toggleSchema(connId, dbName, schemaName) {
    const key = `${connId}-${dbName}-${schemaName}`;
    if (expandedSchemas[key]) {
      delete expandedSchemas[key];
    } else {
      expandedSchemas[key] = true;
      // Set active connection and database when expanding schema
      const conn = $connections.find((c) => c.id === connId);
      const dbData = expandedDatabases[`${connId}-${dbName}`];
      if (conn) {
        activeConnection.set(conn);
      }
      if (dbData?.database) {
        selectedDatabase.set(dbData.database);
      }
    }
    expandedSchemas = { ...expandedSchemas };
  }

  function toggleSchemasParent(connId, dbName) {
    const key = `${connId}-${dbName}`;
    if (expandedSchemasParent[key]) {
      delete expandedSchemasParent[key];
    } else {
      expandedSchemasParent[key] = true;
    }
    expandedSchemasParent = { ...expandedSchemasParent };
  }

  function selectTable(table, connId, dbName) {
    selectedTable.set(table);
    // Ensure active connection and database are set correctly
    if (connId && dbName) {
      const conn = $connections.find((c) => c.id === connId);
      const dbData = expandedDatabases[`${connId}-${dbName}`];
      if (conn) {
        activeConnection.set(conn);
      }
      if (dbData?.database) {
        selectedDatabase.set(dbData.database);
      }
    }
  }

  function handleTableDoubleClick(table, connection, database) {
    // Dispatch event untuk membuka tab baru dengan data tabel
    dispatch("openTableTab", {
      table,
      database: database,
      connection: connection,
    });
  }

  function formatBytes(bytes) {
    if (!bytes || bytes === 0) return "";

    const k = 1024;
    const sizes = ["B", "K", "M", "G", "T"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));

    const value = bytes / Math.pow(k, i);

    // Format: semua angka dibulatkan tanpa desimal
    const formatted = Math.round(value);

    return `${formatted}${sizes[i]}`;
  }

  function openNewConnectionModal() {
    editingConnection = null;
    showModal = true;
  }

  function closeModal() {
    showModal = false;
    editingConnection = null;
  }

  async function handleSaveConnection() {
    closeModal();
    await loadConnections();
  }

  $: filteredConnections = $connections.filter((conn) =>
    conn.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  function handleToggleSidebar() {
    if (onToggleSidebar) {
      onToggleSidebar();
    }
  }

  function handleConnectionContextMenu(event, conn) {
    event.preventDefault();
    event.stopPropagation();
    contextMenu = {
      x: event.clientX,
      y: event.clientY,
      connection: conn,
    };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  function handleEditConnection(conn) {
    editingConnection = conn;
    showModal = true;
    closeContextMenu();
  }

  async function handleDeleteConnection(conn) {
    if (confirm(`Are you sure you want to delete connection "${conn.name}"?`)) {
      try {
        await deleteConnection(conn.id);
        // Remove from connected connections if exists
        if (connectedConnections[conn.id]) {
          delete connectedConnections[conn.id];
          connectedConnections = { ...connectedConnections };
        }
        // Remove from expanded connections if exists
        if (expandedConnections[conn.id]) {
          delete expandedConnections[conn.id];
          expandedConnections = { ...expandedConnections };
        }
        // Reload connections list
        await loadConnections();
        closeContextMenu();
      } catch (error) {
        console.error("Failed to delete connection:", error);
        alert(`Failed to delete connection: ${error}`);
      }
    } else {
      closeContextMenu();
    }
  }

  async function handleRefreshConnection(conn) {
    // Refresh the connection - keep connected status
    if (connectedConnections[conn.id]) {
      delete expandedConnections[conn.id];
      expandedConnections = { ...expandedConnections };
      // Reconnect
      await disconnectFromDatabase(conn.id);
      await toggleConnection(conn);
    }
    closeContextMenu();
  }

  async function handleConnectConnection(conn) {
    // Connect or activate the connection
    activeConnection.set(conn);
    loadingConnections[conn.id] = true;
    loadingConnections = { ...loadingConnections };
    try {
      await connectToDatabase(conn);
      databases = await getDatabases(conn);
      expandedConnections[conn.id] = { databases };
      connectedConnections[conn.id] = true;
      connectedConnections = { ...connectedConnections };
    } catch (error) {
      console.error("Failed to connect:", error);
      alert(`Failed to connect: ${error}`);
    } finally {
      loadingConnections[conn.id] = false;
      loadingConnections = { ...loadingConnections };
    }
    closeContextMenu();
  }

  async function handleDisconnectConnection(conn) {
    // Disconnect - remove from connected list and collapse
    try {
      await disconnectFromDatabase(conn.id);
      delete connectedConnections[conn.id];
      connectedConnections = { ...connectedConnections };
      delete expandedConnections[conn.id];
      expandedConnections = { ...expandedConnections };
    } catch (error) {
      console.error("Failed to disconnect:", error);
      alert(`Failed to disconnect: ${error}`);
    }
    closeContextMenu();
  }

  function handleCopyConnection(conn) {
    // Copy connection details to clipboard
    const text = `${conn.name} (${conn.host}:${conn.port})`;
    navigator.clipboard.writeText(text);
    closeContextMenu();
  }

  function handleContextMenuAction(event) {
    const { type, detail } = event;
    switch (type) {
      case "edit":
        handleEditConnection(detail);
        break;
      case "delete":
        handleDeleteConnection(detail);
        break;
      case "refresh":
        handleRefreshConnection(detail);
        break;
      case "connect":
        handleConnectConnection(detail);
        break;
      case "disconnect":
        handleDisconnectConnection(detail);
        break;
      case "copy":
        handleCopyConnection(detail);
        break;
    }
  }
</script>

<div class="d-flex flex-column h-100 bg-body-tertiary text-dark">
  <div class="p-3 pb-2 border-bottom bg-body">
    <div class="d-flex align-items-center justify-content-between mb-2">
      <h6
        class="text-uppercase text-secondary mb-0"
        style="font-size: 11px; font-weight: 600; letter-spacing: 0.5px;"
      >
        <i class="fas fa-network-wired me-2"></i>
        Connections
      </h6>
      <button
        class="btn btn-sm btn-link text-secondary p-0"
        on:click={handleToggleSidebar}
        title="Hide Sidebar"
        style="width: 20px; height: 20px; font-size: 12px;"
      >
        <i class="fas fa-chevron-left"></i>
      </button>
    </div>
    <div class="d-flex gap-2">
      <input
        type="search"
        class="form-control form-control-sm flex-grow-1"
        placeholder="Search connection or database"
        bind:value={searchQuery}
        style="font-size: 12px;"
      />
      <button
        class="btn btn-sm btn-success"
        on:click={openNewConnectionModal}
        style="font-size: 12px; padding: 4px 8px;"
        title="Add Connection"
      >
        <i class="fas fa-plus"></i>
      </button>
    </div>
  </div>

  <div class="flex-grow-1 overflow-auto p-1" style="scrollbar-width: thin;">
    {#if filteredConnections.length === 0}
      <p class="text-muted small p-3">No connections found</p>
    {/if}

    {#each filteredConnections as conn (conn.id)}
      <div class="tree-item">
        <div class="tree-node connection-node">
          <button
            class="tree-toggle"
            on:click={() => toggleConnection(conn)}
            aria-label="Toggle connection"
          >
            {#if loadingConnections[conn.id]}
              <i class="fas fa-spinner fa-spin"></i>
            {:else}
              <i
                class="fas fa-chevron-{expandedConnections[conn.id]
                  ? 'down'
                  : 'right'}"
              ></i>
            {/if}
          </button>
          <button
            class="tree-label"
            class:active={$activeConnection?.id === conn.id}
            on:click={() => toggleConnection(conn)}
            on:contextmenu={(e) => handleConnectionContextMenu(e, conn)}
          >
            <div class="connection-icon-wrapper">
              {#if conn.db_type === "MySQL"}
                <i class="si si-mysql tree-icon connection-icon"></i>
              {:else if conn.db_type === "PostgreSQL"}
                <i class="si si-postgresql tree-icon connection-icon"></i>
              {:else if conn.db_type === "MongoDB"}
                <i class="si si-mongodb tree-icon connection-icon"></i>
              {:else if conn.db_type === "Redis"}
                <i class="si si-redis tree-icon connection-icon"></i>
              {:else}
                <i class="fas fa-server tree-icon connection-icon"></i>
              {/if}
              {#if connectedConnections[conn.id]}
                <i class="fas fa-check-circle connection-status-badge"></i>
              {/if}
            </div>
            <span class="tree-text">
              {conn.name}
            </span>
            <span class="connection-details">
              <i>{conn.host}:{conn.port}</i>
            </span>
            <span class="tree-badge">
              <!-- { conn.db_type } -->
            </span>
          </button>
        </div>

        {#if expandedConnections[conn.id]}
          <div class="tree-children">
            {#each expandedConnections[conn.id].databases || [] as db (db.name)}
              <div class="tree-item">
                <div class="tree-node database-node">
                  <button
                    class="tree-toggle"
                    on:click={() => toggleDatabase(conn.id, db)}
                    aria-label="Toggle database"
                  >
                    {#if loadingDatabases[`${conn.id}-${db.name}`]}
                      <i class="fas fa-spinner fa-spin"></i>
                    {:else}
                      <i
                        class="fas fa-chevron-{expandedDatabases[
                          `${conn.id}-${db.name}`
                        ]
                          ? 'down'
                          : 'right'}"
                      ></i>
                    {/if}
                  </button>
                  <button
                    class="tree-label"
                    class:active={$selectedDatabase?.name === db.name}
                    on:click={() => toggleDatabase(conn.id, db)}
                  >
                    <i class="fas fa-database tree-icon"></i>
                    <span class="tree-text">{db.name}</span>
                  </button>
                </div>

                {#if expandedDatabases[`${conn.id}-${db.name}`]}
                  <div class="tree-children">
                    {#if conn.db_type === "PostgreSQL"}
                      <!-- PostgreSQL: Schemas Parent -> Individual Schemas -> Tables -->
                      {@const tables =
                        expandedDatabases[`${conn.id}-${db.name}`].tables || []}
                      {@const schemaGroups = tables.reduce((acc, table) => {
                        const schema = table.schema || "public";
                        if (!acc[schema]) acc[schema] = [];
                        acc[schema].push(table);
                        return acc;
                      }, {})}

                      <!-- Schemas Parent -->
                      <div class="tree-item">
                        <div class="tree-node tables-section-node">
                          <button
                            class="tree-toggle"
                            aria-label="Toggle schemas"
                            on:click={() =>
                              toggleSchemasParent(conn.id, db.name)}
                          >
                            <i
                              class="fas fa-chevron-{expandedSchemasParent[
                                `${conn.id}-${db.name}`
                              ]
                                ? 'down'
                                : 'right'}"
                            ></i>
                          </button>
                          <button
                            class="tree-section-header"
                            on:click={() =>
                              toggleSchemasParent(conn.id, db.name)}
                          >
                            <i class="fas fa-folder-tree"></i>
                            <span
                              >Schemas ({Object.keys(schemaGroups)
                                .length})</span
                            >
                          </button>
                        </div>
                        {#if expandedSchemasParent[`${conn.id}-${db.name}`]}
                          <div class="tree-children">
                            {#each Object.entries(schemaGroups) as [schemaName, schemaTables]}
                              <div class="tree-item">
                                <div class="tree-node tables-section-node">
                                  <button
                                    class="tree-toggle"
                                    aria-label="Toggle schema"
                                    on:click={() =>
                                      toggleSchema(
                                        conn.id,
                                        db.name,
                                        schemaName
                                      )}
                                  >
                                    <i
                                      class="fas fa-chevron-{expandedSchemas[
                                        `${conn.id}-${db.name}-${schemaName}`
                                      ]
                                        ? 'down'
                                        : 'right'}"
                                    ></i>
                                  </button>
                                  <button
                                    class="tree-section-header"
                                    on:click={() =>
                                      toggleSchema(
                                        conn.id,
                                        db.name,
                                        schemaName
                                      )}
                                  >
                                    <i class="fas fa-folder"></i>
                                    <span>{schemaName}</span>
                                  </button>
                                </div>
                                {#if expandedSchemas[`${conn.id}-${db.name}-${schemaName}`]}
                                  <div class="tree-children">
                                    <div class="tree-item">
                                      <div
                                        class="tree-node tables-section-node"
                                      >
                                        <button
                                          class="tree-toggle"
                                          aria-label="Toggle tables"
                                          on:click={() =>
                                            toggleTables(
                                              conn.id,
                                              `${db.name}-${schemaName}`
                                            )}
                                        >
                                          <i
                                            class="fas fa-chevron-{expandedTables[
                                              `${conn.id}-${db.name}-${schemaName}`
                                            ]
                                              ? 'down'
                                              : 'right'}"
                                          ></i>
                                        </button>
                                        <button
                                          class="tree-section-header"
                                          on:click={() =>
                                            toggleTables(
                                              conn.id,
                                              `${db.name}-${schemaName}`
                                            )}
                                        >
                                          <i class="fas fa-table"></i>
                                          <span
                                            >Tables ({schemaTables.length})</span
                                          >
                                        </button>
                                      </div>
                                      {#if expandedTables[`${conn.id}-${db.name}-${schemaName}`]}
                                        <div class="tree-children">
                                          <table
                                            class="table table-sm table-hover mb-0 table-borderless"
                                            style="padding-left: 8px;"
                                          >
                                            <tbody>
                                              {#each schemaTables as table (table.name)}
                                                <tr
                                                  class="table-item-row"
                                                  class:table-active={$selectedTable?.name ===
                                                    table.name}
                                                  style="cursor: pointer; line-height: 1.5;"
                                                >
                                                  <td
                                                    class="p-0 align-middle"
                                                    style="width: 100%; max-width: 0; overflow: hidden; white-space: nowrap; padding-left: 8px !important;"
                                                  >
                                                    <button
                                                      class="btn btn-sm p-0 text-secondary"
                                                      style="width: 20px; height: 20px; font-size: 10px; flex-shrink: 0;"
                                                      aria-label="Toggle table"
                                                    >
                                                      <i
                                                        class="fas fa-chevron-right"
                                                      ></i>
                                                    </button>
                                                    <button
                                                      class="btn btn-sm p-1 text-start border-0"
                                                      style="font-size: 12px; display: inline-block; max-width: calc(100% - 24px); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
                                                      on:click={() =>
                                                        selectTable(
                                                          table,
                                                          conn.id,
                                                          db.name
                                                        )}
                                                      on:dblclick={() =>
                                                        handleTableDoubleClick(
                                                          table,
                                                          expandedDatabases[
                                                            `${conn.id}-${db.name}`
                                                          ].connection,
                                                          expandedDatabases[
                                                            `${conn.id}-${db.name}`
                                                          ].database
                                                        )}
                                                    >
                                                      <i
                                                        class="fas fa-table text-secondary me-1"
                                                        style="font-size: 11px;"
                                                      ></i>
                                                      <span
                                                        class="text-truncate"
                                                        title={table.name}
                                                        >{table.name}</span
                                                      >
                                                    </button>
                                                  </td>
                                                  <td
                                                    class="text-end align-middle"
                                                    style="white-space: nowrap; width: 50px; min-width: 50px; max-width: 50px; padding: 2px 8px 2px 4px !important;"
                                                  >
                                                    {#if table.size_bytes !== undefined && table.size_bytes !== null && table.size_bytes > 0}
                                                      <span
                                                        class="badge bg-light text-secondary"
                                                        style="font-size: 10px;"
                                                        title="Table size"
                                                        >{formatBytes(
                                                          table.size_bytes
                                                        )}</span
                                                      >
                                                    {/if}
                                                  </td>
                                                </tr>
                                              {/each}
                                            </tbody>
                                          </table>
                                        </div>
                                      {/if}
                                    </div>
                                  </div>
                                {/if}
                              </div>
                            {/each}
                          </div>
                        {/if}
                      </div>
                    {:else}
                      <!-- MySQL, MongoDB, Redis: Direct Tables/Collections -->
                      <div class="tree-item">
                        <div class="tree-node tables-section-node">
                          <button
                            class="tree-toggle"
                            aria-label="Toggle {conn.db_type === 'MongoDB'
                              ? 'collections'
                              : 'tables'}"
                            on:click={() => toggleTables(conn.id, db.name)}
                          >
                            <i
                              class="fas fa-chevron-{expandedTables[
                                `${conn.id}-${db.name}`
                              ]
                                ? 'down'
                                : 'right'}"
                            ></i>
                          </button>
                          <button
                            class="tree-section-header"
                            on:click={() => toggleTables(conn.id, db.name)}
                          >
                            <i
                              class="fas fa-{conn.db_type === 'MongoDB'
                                ? 'layer-group'
                                : 'table'}"
                            ></i>
                            <span
                              >{conn.db_type === "MongoDB"
                                ? "Collections"
                                : "Tables"} ({expandedDatabases[
                                `${conn.id}-${db.name}`
                              ].tables?.length || 0})</span
                            >
                          </button>
                        </div>
                        {#if expandedTables[`${conn.id}-${db.name}`]}
                          <div class="tree-children">
                            <table
                              class="table table-sm table-hover mb-0 table-borderless"
                              style="padding-left: 8px;"
                            >
                              <tbody>
                                {#each expandedDatabases[`${conn.id}-${db.name}`].tables || [] as table (table.name)}
                                  <tr
                                    class="table-item-row"
                                    class:table-active={$selectedTable?.name ===
                                      table.name}
                                    style="cursor: pointer; line-height: 1.5;"
                                  >
                                    <td
                                      class="p-0 align-middle"
                                      style="width: 100%; max-width: 0; overflow: hidden; white-space: nowrap; padding-left: 8px !important;"
                                    >
                                      <button
                                        class="btn btn-sm p-0 text-secondary"
                                        style="width: 20px; height: 20px; font-size: 10px; flex-shrink: 0;"
                                        aria-label="Toggle {conn.db_type ===
                                        'MongoDB'
                                          ? 'collection'
                                          : 'table'}"
                                      >
                                        <i class="fas fa-chevron-right"></i>
                                      </button>
                                      <button
                                        class="btn btn-sm p-1 text-start border-0"
                                        style="font-size: 12px; display: inline-block; max-width: calc(100% - 24px); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
                                        on:click={() =>
                                          selectTable(table, conn.id, db.name)}
                                        on:dblclick={() =>
                                          handleTableDoubleClick(
                                            table,
                                            expandedDatabases[
                                              `${conn.id}-${db.name}`
                                            ].connection,
                                            expandedDatabases[
                                              `${conn.id}-${db.name}`
                                            ].database
                                          )}
                                      >
                                        <i
                                          class="fas fa-{conn.db_type ===
                                          'MongoDB'
                                            ? 'layer-group'
                                            : 'table'} text-secondary me-1"
                                          style="font-size: 11px;"
                                        ></i>
                                        <span
                                          class="text-truncate"
                                          title={table.name}>{table.name}</span
                                        >
                                      </button>
                                    </td>
                                    <td
                                      class="text-end align-middle"
                                      style="white-space: nowrap; width: 50px; min-width: 50px; max-width: 50px; padding: 2px 8px 2px 4px !important;"
                                    >
                                      {#if table.size_bytes !== undefined && table.size_bytes !== null && table.size_bytes > 0}
                                        <span
                                          class="badge bg-light text-secondary"
                                          style="font-size: 10px;"
                                          title="{conn.db_type === 'MongoDB'
                                            ? 'Collection'
                                            : 'Table'} size"
                                          >{formatBytes(table.size_bytes)}</span
                                        >
                                      {/if}
                                    </td>
                                  </tr>
                                {/each}
                              </tbody>
                            </table>
                          </div>
                        {/if}
                      </div>

                      {#if conn.db_type !== "MongoDB" && conn.db_type !== "Redis"}
                        <div class="tree-item">
                          <div class="tree-node tables-section-node">
                            <button
                              class="tree-toggle"
                              aria-label="Toggle views"
                            >
                              <i class="fas fa-chevron-right"></i>
                            </button>
                            <div class="tree-section-header">
                              <i class="fas fa-eye"></i>
                              <span>Views</span>
                            </div>
                          </div>
                        </div>

                        <div class="tree-item">
                          <div class="tree-node tables-section-node">
                            <button
                              class="tree-toggle"
                              aria-label="Toggle functions"
                            >
                              <i class="fas fa-chevron-right"></i>
                            </button>
                            <div class="tree-section-header">
                              <i class="fas fa-code"></i>
                              <span>Functions</span>
                            </div>
                          </div>
                        </div>
                      {/if}
                    {/if}
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<!-- Context Menu -->
{#if contextMenu}
  <ConnectionContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    connection={contextMenu.connection}
    isConnected={connectedConnections[contextMenu.connection?.id] || false}
    on:edit={handleContextMenuAction}
    on:delete={handleContextMenuAction}
    on:refresh={handleContextMenuAction}
    on:connect={handleContextMenuAction}
    on:disconnect={handleContextMenuAction}
    on:copy={handleContextMenuAction}
  />
{/if}

{#if showModal}
  <ConnectionModal
    connection={editingConnection}
    on:close={closeModal}
    on:save={handleSaveConnection}
  />
{/if}

<style>
  /* Tree structure styles - keeping custom tree functionality */
  .tree-item {
    user-select: none;
  }

  .tree-node {
    display: flex;
    align-items: center;
    gap: 0px;
  }

  .tree-toggle {
    background: transparent;
    border: none;
    color: #6c757d;
    padding: 2px;
    cursor: pointer;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 10px;
    flex-shrink: 0;
    border-radius: 3px;
  }

  .tree-toggle:hover {
    background: #e9ecef;
    color: #212529;
  }

  .tree-label {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 4px;
    background: transparent;
    border: none;
    color: #212529;
    padding: 2px 8px;
    cursor: pointer;
    font-size: 12px;
    text-align: left;
    transition: background-color 0.15s;
    min-height: 20px;
    border-radius: 3px;
    line-height: 1.5;
  }

  .tree-label:hover {
    background: #e9ecef;
  }

  .tree-label.active {
    background: #cfe2ff;
    color: #0d6efd;
    font-weight: 500;
  }

  .tree-icon {
    font-size: 11px;
    width: 16px;
    text-align: center;
    flex-shrink: 0;
    color: #6c757d;
  }

  .connection-icon {
    font-size: 16px !important;
    width: 20px !important;
  }

  .connection-icon-wrapper {
    position: relative;
    display: inline-block;
    flex-shrink: 0;
  }

  .connection-status-badge {
    position: absolute;
    bottom: -2px;
    left: -2px;
    font-size: 8px;
    color: #198754;
    background: white;
    border-radius: 50%;
  }

  .tree-label.active .tree-icon {
    color: #0d6efd;
  }

  .tree-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .connection-details {
    font-size: 12px;
    color: #6c757d;
    font-weight: normal;
    margin-left: 8px;
    flex-shrink: 0;
  }

  .tree-label.active .connection-details {
    color: #0a58ca;
  }

  .tree-badge {
    font-size: 10px;
    padding: 2px 6px;
    background: #f8f9fa;
    border-radius: 3px;
    color: #6c757d;
    font-weight: 500;
    flex-shrink: 0;
    margin-left: auto;
  }

  .tree-label.active .tree-badge {
    background: #b6d4fe;
  }

  .tree-children {
    margin-left: 12px;
  }

  .tree-section-header {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    font-size: 11px;
    color: #6c757d;
    font-weight: 600;
    line-height: 1.5;
    flex: 1;
    background: transparent;
    border: none;
    text-align: left;
    cursor: pointer;
  }

  .tree-section-header:hover {
    background: #e9ecef;
  }

  .tree-section-header i {
    font-size: 11px;
    width: 16px;
    text-align: center;
  }

  .connection-node {
    padding-left: 8px;
  }

  .database-node {
    padding-left: 8px;
  }

  .tables-section-node {
    padding-left: 8px;
  }

  /* Loading spinner animation */
  .fa-spinner {
    color: #0d6efd;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  /* Custom styles for table item active state */
  .table-item-row.table-active .badge {
    background-color: #b6d4fe !important;
  }
</style>
