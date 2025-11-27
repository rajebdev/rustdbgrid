<script>
  import { onMount, createEventDispatcher } from "svelte";
  import {
    connections,
    activeConnection,
    selectedDatabase,
    selectedTable,
  } from "../../../stores/connections";
  import { tabStore } from "../../../stores/tabs";
  import { tabDataStore } from "../../../stores/tabData";
  import {
    getConnections,
    getDatabases,
    getTables,
    deleteConnection,
    saveConnection,
    connectToDatabase,
    disconnectFromDatabase,
    isDatabaseConnected,
    getConnectedDatabases,
  } from "../../../utils/tauri";

  const dispatch = createEventDispatcher();
  import ConnectionModal from "../../modals/ConnectionModal.svelte";
  import ConnectionContextMenu from "../../context-menus/ConnectionContextMenu.svelte";
  import TableContextMenu from "../../context-menus/TableContextMenu.svelte";
  import DatabaseContextMenu from "../../context-menus/DatabaseContextMenu.svelte";
  import SchemaContextMenu from "../../context-menus/SchemaContextMenu.svelte";

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
  let tableContextMenu = null; // { x, y, table, connection, database }
  let databaseContextMenu = null; // { x, y, database, connection }
  let schemaContextMenu = null; // { x, y, schema, database, connection }

  // Track active (right-clicked) items
  let activeContextConnection = null;
  let activeContextDatabase = null;
  let activeContextSchema = null;
  let activeContextTable = null;

  onMount(async () => {
    await loadConnections();
    // Load connected databases from backend
    await syncConnectedStatus();
    // Close context menu when clicking anywhere
    document.addEventListener("click", closeContextMenu);
    document.addEventListener("click", closeTableContextMenu);
    document.addEventListener("click", closeDatabaseContextMenu);
    document.addEventListener("click", closeSchemaContextMenu);
    return () => {
      document.removeEventListener("click", closeContextMenu);
      document.removeEventListener("click", closeTableContextMenu);
      document.removeEventListener("click", closeDatabaseContextMenu);
      document.removeEventListener("click", closeSchemaContextMenu);
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

  function handleConnectionContextMenu(event, conn) {
    event.preventDefault();
    event.stopPropagation();
    // Close all other context menus
    tableContextMenu = null;
    databaseContextMenu = null;
    schemaContextMenu = null;
    // Set active item
    activeContextConnection = conn.id;
    activeContextDatabase = null;
    activeContextSchema = null;
    activeContextTable = null;
    // Show context menu
    contextMenu = {
      x: event.clientX,
      y: event.clientY,
      connection: conn,
    };
  }

  function closeContextMenu() {
    contextMenu = null;
    activeContextConnection = null;
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

      // Close all tabs for this connection
      const closedTabIds = tabStore.closeTabsByConnection(conn.id);
      if (closedTabIds.length > 0) {
        tabDataStore.removeTabsByIds(closedTabIds);
      }

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

  async function handleRenameConnection(conn) {
    // Rename connection
    const newName = prompt(`Rename connection "${conn.name}" to:`, conn.name);
    if (newName && newName !== conn.name) {
      try {
        // Update connection with new name
        const updatedConn = { ...conn, name: newName };
        await saveConnection(updatedConn);
        // Reload connections list
        await loadConnections();
        closeContextMenu();
      } catch (error) {
        console.error("Failed to rename connection:", error);
        alert(`Failed to rename connection: ${error}`);
        closeContextMenu();
      }
    } else {
      closeContextMenu();
    }
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
      case "rename":
        handleRenameConnection(detail);
        break;
    }
  }

  function handleTableContextMenu(event, table, conn, db) {
    event.preventDefault();
    event.stopPropagation();
    // Close all other context menus
    contextMenu = null;
    databaseContextMenu = null;
    schemaContextMenu = null;
    // Set active item
    activeContextConnection = null;
    activeContextDatabase = null;
    activeContextSchema = null;
    activeContextTable = `${conn.id}-${db.name}-${table.schema || "public"}-${table.name}`;
    // Show context menu
    tableContextMenu = {
      x: event.clientX,
      y: event.clientY,
      table: table,
      connection: conn,
      database: db,
    };
  }

  function closeTableContextMenu() {
    tableContextMenu = null;
    activeContextTable = null;
  }

  function handleDatabaseContextMenu(event, db, conn) {
    event.preventDefault();
    event.stopPropagation();
    // Close all other context menus
    contextMenu = null;
    tableContextMenu = null;
    schemaContextMenu = null;
    // Set active item
    activeContextConnection = null;
    activeContextDatabase = `${conn.id}-${db.name}`;
    activeContextSchema = null;
    activeContextTable = null;
    // Show context menu
    databaseContextMenu = {
      x: event.clientX,
      y: event.clientY,
      database: db,
      connection: conn,
    };
  }

  function closeDatabaseContextMenu() {
    databaseContextMenu = null;
    activeContextDatabase = null;
  }

  function handleSchemaContextMenu(event, schemaName, db, conn) {
    event.preventDefault();
    event.stopPropagation();
    // Close all other context menus
    contextMenu = null;
    tableContextMenu = null;
    databaseContextMenu = null;
    // Set active item
    activeContextConnection = null;
    activeContextDatabase = null;
    activeContextSchema = `${conn.id}-${db.name}-${schemaName}`;
    activeContextTable = null;
    // Show context menu
    schemaContextMenu = {
      x: event.clientX,
      y: event.clientY,
      schema: schemaName,
      database: db,
      connection: conn,
    };
  }

  function closeSchemaContextMenu() {
    schemaContextMenu = null;
    activeContextSchema = null;
  }

  function handleSchemaAction(event) {
    const { type, detail } = event;
    const { schema, database, connection } = detail;

    switch (type) {
      case "sqlEditor":
        // Open SQL editor for this schema
        console.log("SQL Editor for schema:", schema);
        dispatch("openSqlEditor", { schema, database, connection });
        break;
      case "viewSchema":
        // View schema properties
        console.log("View Schema:", schema);
        // TODO: Implement view schema
        break;
      case "viewDiagram":
        // View schema diagram
        console.log("View Diagram for schema:", schema);
        // TODO: Implement view diagram
        break;
      case "importData":
        // Import data to schema
        console.log("Import Data to schema:", schema);
        // TODO: Implement import data
        break;
      case "generateSql":
        // Generate SQL for schema
        console.log("Generate SQL for schema:", schema);
        // TODO: Implement generate SQL
        break;
      case "copy":
        // Copy schema name
        navigator.clipboard.writeText(schema);
        break;
      case "paste":
        // Paste action
        console.log("Paste in schema:", schema);
        // TODO: Implement paste
        break;
      case "copyAdvancedInfo":
        // Copy detailed schema info
        const info = `Schema: ${schema}\nDatabase: ${database.name}\nConnection: ${connection.name}\nHost: ${connection.host}:${connection.port}`;
        navigator.clipboard.writeText(info);
        break;
      case "delete":
        // Delete schema
        if (
          confirm(
            `Are you sure you want to delete schema "${schema}"? This action cannot be undone!`
          )
        ) {
          console.log("Delete Schema:", schema);
          // TODO: Implement delete schema
        }
        break;
      case "rename":
        // Rename schema
        const newName = prompt(`Rename schema "${schema}" to:`, schema);
        if (newName && newName !== schema) {
          console.log("Rename Schema:", schema, "to", newName);
          // TODO: Implement rename schema
        }
        break;
      case "refresh":
        // Refresh schema (reload tables)
        const key = `${connection.id}-${database.name}-${schema}`;
        delete expandedSchemas[key];
        expandedSchemas = { ...expandedSchemas };
        toggleSchema(connection.id, database.name, schema);
        break;
    }
    closeSchemaContextMenu();
  }

  function handleDatabaseAction(event) {
    const { type, detail } = event;
    const { database, connection } = detail;

    switch (type) {
      case "sqlEditor":
        // Open SQL editor for this database
        console.log("SQL Editor for database:", database.name);
        dispatch("openSqlEditor", { database, connection });
        break;
      case "create":
        // Create new object in database
        console.log("Create in database:", database.name);
        // TODO: Implement create menu
        break;
      case "viewDatabase":
        // View database properties
        console.log("View Database:", database.name);
        // TODO: Implement view database
        break;
      case "filter":
        // Filter database objects
        console.log("Filter database:", database.name);
        // TODO: Implement filter
        break;
      case "compareMigrate":
        // Compare/migrate database
        console.log("Compare/Migrate:", database.name);
        // TODO: Implement compare/migrate
        break;
      case "tools":
        // Database tools
        console.log("Tools for database:", database.name);
        // TODO: Implement tools menu
        break;
      case "copy":
        // Copy database name
        navigator.clipboard.writeText(database.name);
        break;
      case "paste":
        // Paste action
        console.log("Paste in database:", database.name);
        // TODO: Implement paste
        break;
      case "copyAdvancedInfo":
        // Copy detailed database info
        const info = `Database: ${database.name}\nConnection: ${connection.name}\nHost: ${connection.host}:${connection.port}`;
        navigator.clipboard.writeText(info);
        break;
      case "delete":
        // Delete database
        if (
          confirm(
            `Are you sure you want to delete database "${database.name}"? This action cannot be undone!`
          )
        ) {
          console.log("Delete Database:", database.name);
          // TODO: Implement delete database
        }
        break;
      case "rename":
        // Rename database
        const newName = prompt(
          `Rename database "${database.name}" to:`,
          database.name
        );
        if (newName && newName !== database.name) {
          console.log("Rename Database:", database.name, "to", newName);
          // TODO: Implement rename database
        }
        break;
      case "refresh":
        // Refresh database (reload tables)
        const key = `${connection.id}-${database.name}`;
        delete expandedDatabases[key];
        expandedDatabases = { ...expandedDatabases };
        toggleDatabase(connection.id, database);
        break;
    }
    closeDatabaseContextMenu();
  }

  function handleTableAction(event) {
    const { type, detail } = event;
    const { table, connection, database } = detail;

    switch (type) {
      case "viewTable":
        // Open table structure view
        console.log("View Table:", table.name);
        // TODO: Implement view table structure
        break;
      case "viewDiagram":
        // Open ER diagram
        console.log("View Diagram:", table.name);
        // TODO: Implement view diagram
        break;
      case "viewData":
        // Open table data view - same as double click
        handleTableDoubleClick(table, connection, database);
        break;
      case "exportData":
        // Export table data
        console.log("Export Data:", table.name);
        // TODO: Implement export data
        break;
      case "importData":
        // Import data to table
        console.log("Import Data:", table.name);
        // TODO: Implement import data
        break;
      case "readInConsole":
        // Generate SELECT query in SQL console
        console.log("Read in Console:", table.name);
        // TODO: Implement read in SQL console
        break;
      case "copy":
        // Copy table name
        navigator.clipboard.writeText(table.name);
        break;
      case "copyAdvancedInfo":
        // Copy detailed table info
        const info = `Table: ${table.name}\nDatabase: ${database.name}\nConnection: ${connection.name}`;
        navigator.clipboard.writeText(info);
        break;
      case "delete":
        // Delete table
        if (confirm(`Are you sure you want to delete table "${table.name}"?`)) {
          console.log("Delete Table:", table.name);
          // TODO: Implement delete table
        }
        break;
      case "rename":
        // Rename table
        const newName = prompt(`Rename table "${table.name}" to:`, table.name);
        if (newName && newName !== table.name) {
          console.log("Rename Table:", table.name, "to", newName);
          // TODO: Implement rename table
        }
        break;
      case "refresh":
        // Refresh table list
        toggleDatabase(connection.id, database);
        break;
    }
    closeTableContextMenu();
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
            class:active={$activeConnection?.id === conn.id ||
              activeContextConnection === conn.id}
            on:click={() => toggleConnection(conn)}
            on:contextmenu={(e) => handleConnectionContextMenu(e, conn)}
          >
            <div class="connection-icon-wrapper">
              {#if conn.db_type === "MySQL"}
                <span class="tree-icon connection-icon db-emoji">🐬</span>
              {:else if conn.db_type === "PostgreSQL"}
                <span class="tree-icon connection-icon db-emoji">🐘</span>
              {:else if conn.db_type === "MongoDB"}
                <span class="tree-icon connection-icon db-emoji">🍃</span>
              {:else if conn.db_type === "Redis"}
                <i class="fas fa-database tree-icon connection-icon redis-icon"
                ></i>
              {:else if conn.db_type === "Ignite"}
                <span class="tree-icon connection-icon db-emoji">🔥</span>
              {:else if conn.db_type === "MSSQL"}
                <span class="tree-icon connection-icon db-emoji">🗄️</span>
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
            {#if conn.db_type === "Ignite"}
              <!-- Ignite: Direct Caches (no database level) -->
              {@const caches = expandedConnections[conn.id].databases || []}
              <table
                class="table table-sm table-hover mb-0 table-borderless"
                style="padding-left: 8px;"
              >
                <tbody>
                  {#each caches as cache (cache.name)}
                    <tr
                      class="table-item-row"
                      class:table-active={$selectedTable?.name === cache.name}
                      style="cursor: pointer; line-height: 1.5;"
                    >
                      <td
                        class="p-0 align-middle"
                        style="width: 100%; max-width: 0; overflow: hidden; white-space: nowrap; padding-left: 8px !important;"
                      >
                        <button
                          class="btn btn-sm p-0 text-secondary"
                          style="width: 20px; height: 20px; font-size: 10px; flex-shrink: 0;"
                          aria-label="Cache"
                        >
                          <i class="fas fa-chevron-right"></i>
                        </button>
                        <button
                          class="btn btn-sm p-1 text-start border-0"
                          style="font-size: 12px; display: inline-block; max-width: calc(100% - 24px); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
                          on:click={() => {
                            const table = {
                              name: cache.name,
                              schema: null,
                              size_bytes: null,
                            };
                            selectTable(table, conn.id, cache.name);
                          }}
                          on:dblclick={() => {
                            const table = {
                              name: cache.name,
                              schema: null,
                              size_bytes: null,
                            };
                            handleTableDoubleClick(table, conn, {
                              name: cache.name,
                            });
                          }}
                          on:contextmenu={(e) => {
                            const table = {
                              name: cache.name,
                              schema: null,
                              size_bytes: null,
                            };
                            handleTableContextMenu(e, table, conn, {
                              name: cache.name,
                            });
                          }}
                        >
                          <i
                            class="fas fa-server text-secondary me-1"
                            style="font-size: 11px;"
                          ></i>
                          <span class="text-truncate" title={cache.name}
                            >{cache.name}</span
                          >
                        </button>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            {:else}
              <!-- Other databases: normal hierarchy -->
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
                      class:active={$selectedDatabase?.name === db.name ||
                        activeContextDatabase === `${conn.id}-${db.name}`}
                      on:click={() => toggleDatabase(conn.id, db)}
                      on:contextmenu={(e) =>
                        handleDatabaseContextMenu(e, db, conn)}
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
                          expandedDatabases[`${conn.id}-${db.name}`].tables ||
                          []}
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
                                      class:active={activeContextSchema ===
                                        `${conn.id}-${db.name}-${schemaName}`}
                                      on:click={() =>
                                        toggleSchema(
                                          conn.id,
                                          db.name,
                                          schemaName
                                        )}
                                      on:contextmenu={(e) =>
                                        handleSchemaContextMenu(
                                          e,
                                          schemaName,
                                          db,
                                          conn
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
                                                      table.name ||
                                                      activeContextTable ===
                                                        `${conn.id}-${db.name}-${schemaName}-${table.name}`}
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
                                                        on:contextmenu={(e) =>
                                                          handleTableContextMenu(
                                                            e,
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
                      {:else if conn.db_type === "MSSQL"}
                        <!-- MSSQL: Direct Schemas (dbo, etc.) -> Tables -->
                        {@const tables =
                          expandedDatabases[`${conn.id}-${db.name}`].tables ||
                          []}
                        {@const schemaGroups = tables.reduce((acc, table) => {
                          const schema = table.schema || "dbo";
                          if (!acc[schema]) acc[schema] = [];
                          acc[schema].push(table);
                          return acc;
                        }, {})}

                        {#each Object.entries(schemaGroups) as [schemaName, schemaTables]}
                          <div class="tree-item">
                            <div class="tree-node tables-section-node">
                              <button
                                class="tree-toggle"
                                aria-label="Toggle schema"
                                on:click={() =>
                                  toggleSchema(conn.id, db.name, schemaName)}
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
                                class:active={activeContextSchema ===
                                  `${conn.id}-${db.name}-${schemaName}`}
                                on:click={() =>
                                  toggleSchema(conn.id, db.name, schemaName)}
                                on:contextmenu={(e) =>
                                  handleSchemaContextMenu(
                                    e,
                                    schemaName,
                                    db,
                                    conn
                                  )}
                              >
                                <i class="fas fa-database"></i>
                                <span>{schemaName}</span>
                              </button>
                            </div>
                            {#if expandedSchemas[`${conn.id}-${db.name}-${schemaName}`]}
                              <div class="tree-children">
                                <div class="tree-item">
                                  <div class="tree-node tables-section-node">
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
                                      <span>Tables ({schemaTables.length})</span
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
                                                table.name ||
                                                activeContextTable ===
                                                  `${conn.id}-${db.name}-${schemaName}-${table.name}`}
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
                                                  on:contextmenu={(e) =>
                                                    handleTableContextMenu(
                                                      e,
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
                      {:else}
                        <!-- MySQL, MongoDB, Redis, Ignite: Direct Tables/Collections/Caches -->
                        <div class="tree-item">
                          <div class="tree-node tables-section-node">
                            <button
                              class="tree-toggle"
                              aria-label="Toggle {conn.db_type === 'MongoDB'
                                ? 'collections'
                                : conn.db_type === 'Ignite'
                                  ? 'caches'
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
                                  : conn.db_type === 'Ignite'
                                    ? 'server'
                                    : 'table'}"
                              ></i>
                              <span
                                >{conn.db_type === "MongoDB"
                                  ? "Collections"
                                  : conn.db_type === "Ignite"
                                    ? "Caches"
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
                                        table.name ||
                                        activeContextTable ===
                                          `${conn.id}-${db.name}-${table.schema || "public"}-${table.name}`}
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
                                            : conn.db_type === 'Ignite'
                                              ? 'cache'
                                              : 'table'}"
                                        >
                                          <i class="fas fa-chevron-right"></i>
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
                                          on:contextmenu={(e) =>
                                            handleTableContextMenu(
                                              e,
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
                                              : conn.db_type === 'Ignite'
                                                ? 'server'
                                                : 'table'} text-secondary me-1"
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
                                            title="{conn.db_type === 'MongoDB'
                                              ? 'Collection'
                                              : conn.db_type === 'Ignite'
                                                ? 'Cache'
                                                : 'Table'} size"
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

                        {#if conn.db_type !== "MongoDB" && conn.db_type !== "Redis" && conn.db_type !== "Ignite"}
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
            {/if}
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<!-- Context Menu -->
{#if contextMenu}
  {#key contextMenu.connection?.id}
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
      on:rename={handleContextMenuAction}
    />
  {/key}
{/if}

<!-- Table Context Menu -->
{#if tableContextMenu}
  {#key `${tableContextMenu.connection?.id}-${tableContextMenu.database?.name}-${tableContextMenu.table?.name}`}
    <TableContextMenu
      x={tableContextMenu.x}
      y={tableContextMenu.y}
      table={tableContextMenu.table}
      connection={tableContextMenu.connection}
      database={tableContextMenu.database}
      on:viewTable={handleTableAction}
      on:viewDiagram={handleTableAction}
      on:viewData={handleTableAction}
      on:exportData={handleTableAction}
      on:importData={handleTableAction}
      on:readInConsole={handleTableAction}
      on:copy={handleTableAction}
      on:copyAdvancedInfo={handleTableAction}
      on:delete={handleTableAction}
      on:rename={handleTableAction}
      on:refresh={handleTableAction}
    />
  {/key}
{/if}

{#if databaseContextMenu}
  {#key `${databaseContextMenu.connection?.id}-${databaseContextMenu.database?.name}`}
    <DatabaseContextMenu
      x={databaseContextMenu.x}
      y={databaseContextMenu.y}
      database={databaseContextMenu.database}
      connection={databaseContextMenu.connection}
      on:sqlEditor={handleDatabaseAction}
      on:create={handleDatabaseAction}
      on:viewDatabase={handleDatabaseAction}
      on:filter={handleDatabaseAction}
      on:compareMigrate={handleDatabaseAction}
      on:tools={handleDatabaseAction}
      on:copy={handleDatabaseAction}
      on:paste={handleDatabaseAction}
      on:copyAdvancedInfo={handleDatabaseAction}
      on:delete={handleDatabaseAction}
      on:rename={handleDatabaseAction}
      on:refresh={handleDatabaseAction}
    />
  {/key}
{/if}

{#if schemaContextMenu}
  {#key `${schemaContextMenu.connection?.id}-${schemaContextMenu.database?.name}-${schemaContextMenu.schema}`}
    <SchemaContextMenu
      x={schemaContextMenu.x}
      y={schemaContextMenu.y}
      schema={schemaContextMenu.schema}
      database={schemaContextMenu.database}
      connection={schemaContextMenu.connection}
      on:sqlEditor={handleSchemaAction}
      on:viewSchema={handleSchemaAction}
      on:viewDiagram={handleSchemaAction}
      on:importData={handleSchemaAction}
      on:generateSql={handleSchemaAction}
      on:copy={handleSchemaAction}
      on:paste={handleSchemaAction}
      on:copyAdvancedInfo={handleSchemaAction}
      on:delete={handleSchemaAction}
      on:rename={handleSchemaAction}
      on:refresh={handleSchemaAction}
    />
  {/key}
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

  /* Database emoji icons */
  .db-emoji {
    font-style: normal;
    font-size: 16px;
    line-height: 1;
  }

  .redis-icon {
    color: #dc382d !important;
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
    border-radius: 3px;
  }

  .tree-section-header:hover {
    background: #e9ecef;
  }

  .tree-section-header.active {
    background: #cfe2ff;
    color: #0d6efd;
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
