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
    getConnectionsInfo,
    getDatabaseObject,
    deleteConnection,
    saveConnection,
    disconnectFromDatabase,
    getConnectedDatabases,
  } from "../../../utils/tauri";

  const dispatch = createEventDispatcher();
  import ConnectionModal from "../../modals/ConnectionModal.svelte";
  import InputModal from "../../modals/InputModal.svelte";
  import ConnectionContextMenu from "../../context-menus/ConnectionContextMenu.svelte";
  import TableContextMenu from "../../context-menus/TableContextMenu.svelte";
  import DatabaseContextMenu from "../../context-menus/DatabaseContextMenu.svelte";
  import SchemaContextMenu from "../../context-menus/SchemaContextMenu.svelte";
  import ViewContextMenu from "../../context-menus/ViewContextMenu.svelte";

  let databases = [];
  let showModal = false;
  let editingConnection = null;
  let showRenameModal = false;
  let renameModalTitle = "";
  let renameModalValue = "";
  let renameModalCallback = null;
  let expandedConnections = {};
  let expandedDatabases = {};
  let expandedTables = {};
  let expandedSchemas = {};
  let expandedSchemasParent = {};
  let expandedViews = {};
  let expandedIndexes = {};
  let expandedProcedures = {};
  let expandedTriggers = {};
  let expandedEvents = {};
  let searchQuery = "";
  let loadingConnections = {}; // Track loading state per connection
  let loadingDatabases = {}; // Track loading state per database
  let loadingViews = {};
  let loadingIndexes = {};
  let loadingProcedures = {};
  let loadingTriggers = {};
  let loadingEvents = {};
  let loadingSchemas = {}; // Track loading state for schema object counts
  let loadingTables = {}; // Track loading state for tables
  // Store counts for MySQL objects (loaded when database expands)
  let dbObjectCounts = {}; // { 'connId-dbName': { views: n, indexes: n, procedures: n, triggers: n, events: n } }
  // Cache for MySQL objects data (loaded when database expands)
  let cachedTables = {};
  let cachedViews = {};
  let cachedIndexes = {};
  let cachedProcedures = {};
  let cachedTriggers = {};
  let cachedEvents = {};
  // Store counts for PostgreSQL/MSSQL schema objects (loaded when schema expands)
  let schemaObjectCounts = {}; // { 'connId-dbName-schemaName': { views: n, indexes: n, procedures: n, triggers: n, events: n } }
  // Cache for PostgreSQL/MSSQL schema objects data
  let cachedSchemaTables = {};
  let cachedSchemaViews = {};
  let cachedSchemaIndexes = {};
  let cachedSchemaProcedures = {};
  let cachedSchemaTriggers = {};
  let connectedConnections = {}; // Track connection status
  let cachedSchemasParent = {}; // Track if schemas parent is connected
  let contextMenu = null; // { x, y, connection }
  let tableContextMenu = null; // { x, y, table, connection, database }
  let databaseContextMenu = null; // { x, y, database, connection }
  let schemaContextMenu = null; // { x, y, schema, database, connection }
  let viewContextMenu = null; // { x, y, view, connection, database }

  // Track active (right-clicked) items
  let activeContextConnection = null;
  let activeContextDatabase = null;
  let activeContextSchema = null;
  let activeContextTable = null;
  let activeContextView = null;

  onMount(async () => {
    console.log("[FRONTEND] Sidebar onMount starting...");
    await loadConnections();
    console.log("[FRONTEND] Connections loaded");
    // Load connected databases from backend
    await syncConnectedStatus();
    console.log("[FRONTEND] Connected status synced");
    // Close context menu when clicking anywhere
    document.addEventListener("click", closeContextMenu);
    document.addEventListener("click", closeTableContextMenu);
    document.addEventListener("click", closeDatabaseContextMenu);
    document.addEventListener("click", closeSchemaContextMenu);
    document.addEventListener("click", closeViewContextMenu);
    return () => {
      document.removeEventListener("click", closeContextMenu);
      document.removeEventListener("click", closeTableContextMenu);
      document.removeEventListener("click", closeDatabaseContextMenu);
      document.removeEventListener("click", closeSchemaContextMenu);
      document.removeEventListener("click", closeViewContextMenu);
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
      const conns = await getConnectionsInfo();
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
        // Use new unified API
        const result = await getDatabaseObject(conn.id, "database_list");
        databases = result.databases || [];

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

      // For PostgreSQL, use dedicated handler
      if (conn && conn.db_type === "PostgreSQL") {
        await togglePostgreSQLDatabase(connId, db);
      } else {
        // For other databases (MySQL, MSSQL)
        await toggleOtherDatabase(connId, db);
      }
    } else {
      delete expandedDatabases[key];
    }
    expandedDatabases = { ...expandedDatabases };
  }

  async function togglePostgreSQLDatabase(connId, db) {
    const key = `${connId}-${db.name}`;
    const conn = $connections.find((c) => c.id === connId);

    loadingDatabases[key] = true;
    loadingDatabases = { ...loadingDatabases };
    try {
      // Get list of schemas for this database
      const result = await getDatabaseObject(connId, "schema_list", db.name);
      const schemas = result.schemas || [];

      // Set expandedDatabases
      expandedDatabases[key] = {
        schemas: schemas,
        tables: [], // Will be populated when schemas are expanded
        connection: conn,
        database: db,
      };
      expandedDatabases = { ...expandedDatabases };

      // Cache schemas only, don't auto-expand
      cachedSchemasParent[key] = schemas;
    } catch (error) {
      console.error("Failed to load PostgreSQL database:", error);
    } finally {
      loadingDatabases[key] = false;
      loadingDatabases = { ...loadingDatabases };
    }
  }

  async function toggleOtherDatabase(connId, db) {
    const key = `${connId}-${db.name}`;
    const conn = $connections.find((c) => c.id === connId);

    loadingDatabases[key] = true;
    loadingDatabases = { ...loadingDatabases };
    try {
      // For MySQL/MSSQL, use database_info
      const result = await getDatabaseObject(connId, "database_info", db.name);

      // Extract all lists from result
      const tables = result.tables || [];
      const views = result.views || [];
      const indexes = result.indexes || [];
      const procedures = result.procedures || [];
      const triggers = result.triggers || [];
      const events = result.events || [];

      // Store counts for display
      dbObjectCounts[key] = {
        tables: tables.length,
        views: views.length,
        indexes: indexes.length,
        procedures: procedures.length,
        triggers: triggers.length,
        events: events.length,
      };
      dbObjectCounts = { ...dbObjectCounts };

      // Cache the loaded data
      cachedTables[key] = tables;
      cachedViews[key] = views;
      cachedIndexes[key] = indexes;
      cachedProcedures[key] = procedures;
      cachedTriggers[key] = triggers;
      cachedEvents[key] = events;

      // For MSSQL, use tables for schema grouping
      let finalTables = [];
      if (conn && conn.db_type === "MSSQL") {
        finalTables = tables;
      }

      // Set expandedDatabases with tables
      expandedDatabases[key] = {
        tables: finalTables,
        connection: conn,
        database: db,
      };
      expandedDatabases = { ...expandedDatabases };
    } catch (error) {
      console.error("Failed to load database info:", error);
    } finally {
      loadingDatabases[key] = false;
      loadingDatabases = { ...loadingDatabases };
    }
  }

  async function toggleTables(connId, dbName) {
    const key = `${connId}-${dbName}`;
    if (expandedTables[key]) {
      delete expandedTables[key];
      expandedTables = { ...expandedTables };
    } else {
      // Use cached data from database_info
      const tables = cachedTables[key] || [];
      expandedTables[key] = { tables };
      expandedTables = { ...expandedTables };
    }
  }

  async function toggleViews(connId, dbName) {
    const key = `${connId}-${dbName}`;
    if (expandedViews[key]) {
      delete expandedViews[key];
      expandedViews = { ...expandedViews };
    } else {
      // Use cached data from database_info
      const views = cachedViews[key] || [];
      expandedViews[key] = { views };
      expandedViews = { ...expandedViews };
    }
  }

  async function toggleIndexes(connId, dbName) {
    const key = `${connId}-${dbName}`;
    if (expandedIndexes[key]) {
      delete expandedIndexes[key];
      expandedIndexes = { ...expandedIndexes };
    } else {
      // Use cached data from database_info
      const indexes = cachedIndexes[key] || [];
      expandedIndexes[key] = { indexes };
      expandedIndexes = { ...expandedIndexes };
    }
  }

  async function toggleProcedures(connId, dbName) {
    const key = `${connId}-${dbName}`;
    if (expandedProcedures[key]) {
      delete expandedProcedures[key];
      expandedProcedures = { ...expandedProcedures };
    } else {
      // Use cached data from database_info
      const procedures = cachedProcedures[key] || [];
      expandedProcedures[key] = { procedures };
      expandedProcedures = { ...expandedProcedures };
    }
  }

  async function toggleTriggers(connId, dbName) {
    const key = `${connId}-${dbName}`;
    if (expandedTriggers[key]) {
      delete expandedTriggers[key];
      expandedTriggers = { ...expandedTriggers };
    } else {
      // Use cached data from database_info
      const triggers = cachedTriggers[key] || [];
      expandedTriggers[key] = { triggers };
      expandedTriggers = { ...expandedTriggers };
    }
  }

  async function toggleEvents(connId, dbName) {
    const key = `${connId}-${dbName}`;
    if (expandedEvents[key]) {
      delete expandedEvents[key];
      expandedEvents = { ...expandedEvents };
    } else {
      // Use cached data from database_info
      const events = cachedEvents[key] || [];
      expandedEvents[key] = { events };
      expandedEvents = { ...expandedEvents };
    }
  }

  async function toggleSchema(connId, dbName, schemaName) {
    const key = `${connId}-${dbName}-${schemaName}`;
    if (expandedSchemas[key]) {
      delete expandedSchemas[key];
      expandedSchemas = { ...expandedSchemas };
    } else {
      // Set active connection and database when expanding schema
      const conn = $connections.find((c) => c.id === connId);
      const dbData = expandedDatabases[`${connId}-${dbName}`];
      if (conn) {
        activeConnection.set(conn);
      }
      if (dbData?.database) {
        selectedDatabase.set(dbData.database);
      }

      // Load PostgreSQL/MSSQL schema objects if not already loaded
      if (conn && (conn.db_type === "PostgreSQL" || conn.db_type === "MSSQL")) {
        // Check if data is already cached
        const hasData = cachedSchemaViews[key] !== undefined;

        if (!hasData) {
          // Set loading state
          loadingSchemas[key] = true;
          loadingSchemas = { ...loadingSchemas };

          try {
            // Load schema_info to get full lists
            const result = await getDatabaseObject(
              connId,
              "schema_info",
              dbName,
              schemaName
            );

            // Extract all lists from result
            const tables = result.tables || [];
            const views = result.views || [];
            const indexes = result.indexes || [];
            const procedures = result.procedures || [];
            const triggers = result.triggers || [];

            // Store counts for display
            schemaObjectCounts[key] = {
              tables: tables.length,
              views: views.length,
              indexes: indexes.length,
              procedures: procedures.length,
              triggers: triggers.length,
            };
            schemaObjectCounts = { ...schemaObjectCounts };

            // Cache the loaded data
            cachedSchemaTables[key] = tables;
            cachedSchemaViews[key] = views;
            cachedSchemaIndexes[key] = indexes;
            cachedSchemaProcedures[key] = procedures;
            cachedSchemaTriggers[key] = triggers;

            // Trigger reactivity
            cachedSchemaTables = { ...cachedSchemaTables };
            cachedSchemaViews = { ...cachedSchemaViews };
            cachedSchemaIndexes = { ...cachedSchemaIndexes };
            cachedSchemaProcedures = { ...cachedSchemaProcedures };
            cachedSchemaTriggers = { ...cachedSchemaTriggers };
          } catch (error) {
            console.error(
              `Failed to load schema info for ${schemaName}:`,
              error
            );
            // Set empty caches on error
            cachedSchemaTables[key] = [];
            cachedSchemaViews[key] = [];
            cachedSchemaIndexes[key] = [];
            cachedSchemaProcedures[key] = [];
            cachedSchemaTriggers[key] = [];
          } finally {
            // Clear loading state
            loadingSchemas[key] = false;
            loadingSchemas = { ...loadingSchemas };
          }
        }
      }

      // Expand schema AFTER data is loaded
      expandedSchemas[key] = true;
      expandedSchemas = { ...expandedSchemas };
    }
  }

  function toggleSchemasParent(connId, dbName) {
    const key = `${connId}-${dbName}`;
    if (expandedSchemasParent[key]) {
      delete expandedSchemasParent[key];
    } else {
      // Use cached data
      const schemas = cachedSchemasParent[key] || [];
      expandedSchemasParent[key] = { schemas };
    }
    expandedSchemasParent = { ...expandedSchemasParent };
  }

  // Schema-level toggle functions for PostgreSQL/MSSQL
  async function toggleSchemaTables(connId, dbName, schemaName) {
    const key = `${connId}-${dbName}-${schemaName}`;
    if (expandedTables[key]) {
      delete expandedTables[key];
      expandedTables = { ...expandedTables };
    } else {
      // Use cached data from schema_info
      const tables = cachedSchemaTables[key] || [];
      expandedTables[key] = { tables };
      expandedTables = { ...expandedTables };
    }
  }

  // Schema-level toggle functions for PostgreSQL/MSSQL
  async function toggleSchemaViews(connId, dbName, schemaName) {
    const key = `${connId}-${dbName}-${schemaName}`;
    if (expandedViews[key]) {
      delete expandedViews[key];
      expandedViews = { ...expandedViews };
    } else {
      // Use cached data from schema_info
      const views = cachedSchemaViews[key] || [];
      expandedViews[key] = { views };
      expandedViews = { ...expandedViews };
    }
  }

  async function toggleSchemaIndexes(connId, dbName, schemaName) {
    const key = `${connId}-${dbName}-${schemaName}`;
    if (expandedIndexes[key]) {
      delete expandedIndexes[key];
      expandedIndexes = { ...expandedIndexes };
    } else {
      // Use cached data from schema_info
      const indexes = cachedSchemaIndexes[key] || [];
      expandedIndexes[key] = { indexes };
      expandedIndexes = { ...expandedIndexes };
    }
  }

  async function toggleSchemaProcedures(connId, dbName, schemaName) {
    const key = `${connId}-${dbName}-${schemaName}`;
    if (expandedProcedures[key]) {
      delete expandedProcedures[key];
      expandedProcedures = { ...expandedProcedures };
    } else {
      // Use cached data from schema_info
      const procedures = cachedSchemaProcedures[key] || [];
      expandedProcedures[key] = { procedures };
      expandedProcedures = { ...expandedProcedures };
    }
  }

  async function toggleSchemaTriggers(connId, dbName, schemaName) {
    const key = `${connId}-${dbName}-${schemaName}`;
    if (expandedTriggers[key]) {
      delete expandedTriggers[key];
      expandedTriggers = { ...expandedTriggers };
    } else {
      // Use cached data from schema_info
      const triggers = cachedSchemaTriggers[key] || [];
      expandedTriggers[key] = { triggers };
      expandedTriggers = { ...expandedTriggers };
    }
  }

  function selectTable(table, connId, dbName, schemaName = null) {
    // Store table with full parent hierarchy for proper identification
    selectedTable.set({
      ...table,
      _connId: connId,
      _dbName: dbName,
      _schema: schemaName || table.schema || null,
    });
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
    // Clear all active context highlights
    activeContextConnection = null;
    activeContextDatabase = null;
    activeContextSchema = null;
    activeContextTable = null;

    // Dispatch event untuk membuka tab baru dengan data tabel
    dispatch("openTableTab", {
      table,
      database: database,
      connection: connection,
    });
  }

  function handleViewDoubleClick(view, connection, database) {
    // Clear all active context highlights
    activeContextConnection = null;
    activeContextDatabase = null;
    activeContextSchema = null;
    activeContextTable = null;

    // Dispatch event untuk membuka tab baru dengan data view (sama seperti tabel)
    dispatch("openTableTab", {
      table: { name: view.name, schema: view.schema, isView: true },
      database: database,
      connection: connection,
    });
  }

  function handleProcedureDoubleClick(
    proc,
    connection,
    database,
    schemaName = null
  ) {
    // Clear all active context highlights
    activeContextConnection = null;
    activeContextDatabase = null;
    activeContextSchema = null;
    activeContextTable = null;

    // Dispatch event untuk membuka tab baru dengan source procedure
    // Ensure schema is set in procedure object
    const procedureWithSchema = {
      ...proc,
      schema: schemaName || proc.schema,
    };
    dispatch("openProcedureTab", {
      procedure: procedureWithSchema,
      database: database,
      connection: connection,
    });
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
      const result = await getDatabaseObject(conn.id, "database_list");
      const databases = result.databases || [];
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
    renameModalTitle = "Rename Connection";
    renameModalValue = conn.name;
    renameModalCallback = async (newName) => {
      if (newName && newName !== conn.name) {
        try {
          // Update connection with new name
          const updatedConn = { ...conn, name: newName };
          await saveConnection(updatedConn);
          // Reload connections list
          await loadConnections();
        } catch (error) {
          console.error("Failed to rename connection:", error);
          alert(`Failed to rename connection: ${error}`);
        }
      }
    };
    showRenameModal = true;
    closeContextMenu();
  }

  function handleRenameSubmit(event) {
    const newName = event.detail;
    if (renameModalCallback) {
      renameModalCallback(newName).catch((error) => {
        console.error("Failed to rename:", error);
      });
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
        renameModalTitle = "Rename Schema";
        renameModalValue = schema;
        renameModalCallback = async (newName) => {
          if (newName && newName !== schema) {
            console.log("Rename Schema:", schema, "to", newName);
            // TODO: Implement rename schema
          }
        };
        showRenameModal = true;
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
        renameModalTitle = "Rename Database";
        renameModalValue = database.name;
        renameModalCallback = async (newName) => {
          if (newName && newName !== database.name) {
            console.log("Rename Database:", database.name, "to", newName);
            // TODO: Implement rename database
          }
        };
        showRenameModal = true;
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
        renameModalTitle = "Rename Table";
        renameModalValue = table.name;
        renameModalCallback = async (newName) => {
          if (newName && newName !== table.name) {
            console.log("Rename Table:", table.name, "to", newName);
            // TODO: Implement rename table
          }
        };
        showRenameModal = true;
        break;
      case "refresh":
        // Refresh table list
        toggleDatabase(connection.id, database);
        break;
    }
    closeTableContextMenu();
  }

  function handleViewContextMenu(event, view, conn, db) {
    event.preventDefault();
    event.stopPropagation();
    // Close all other context menus
    contextMenu = null;
    tableContextMenu = null;
    databaseContextMenu = null;
    schemaContextMenu = null;
    // Set active item
    activeContextConnection = null;
    activeContextDatabase = null;
    activeContextSchema = null;
    activeContextTable = null;
    activeContextView = `${conn.id}-${db.name}-${view.schema || "public"}-${view.name}`;
    // Show context menu
    viewContextMenu = {
      x: event.clientX,
      y: event.clientY,
      view: view,
      connection: conn,
      database: db,
    };
  }

  function closeViewContextMenu() {
    viewContextMenu = null;
    activeContextView = null;
  }

  function handleViewAction(event) {
    const { type, detail } = event;
    const { view, connection, database } = detail;

    switch (type) {
      case "viewStructure":
        // Open view structure
        console.log("View Structure:", view.name);
        // TODO: Implement view structure
        break;
      case "viewDefinition":
        // Open view definition (SQL source)
        console.log("View Definition:", view.name);
        // TODO: Implement view definition
        break;
      case "viewData":
        // Open view data - same as double click
        handleViewDoubleClick(view, connection, database);
        break;
      case "exportData":
        // Export view data
        console.log("Export Data:", view.name);
        // TODO: Implement export data
        break;
      case "importData":
        // Import data to view
        console.log("Import Data:", view.name);
        // TODO: Implement import data
        break;
      case "readInConsole":
        // Generate SELECT query in SQL console
        console.log("Read in Console:", view.name);
        // TODO: Implement read in SQL console
        break;
      case "copy":
        // Copy view name
        navigator.clipboard.writeText(view.name);
        break;
      case "copyAdvancedInfo":
        // Copy detailed view info
        const info = `View: ${view.name}\nDatabase: ${database.name}\nConnection: ${connection.name}`;
        navigator.clipboard.writeText(info);
        break;
      case "delete":
        // Delete view
        if (confirm(`Are you sure you want to delete view "${view.name}"?`)) {
          console.log("Delete View:", view.name);
          // TODO: Implement delete view
        }
        break;
      case "rename":
        // Rename view
        renameModalTitle = "Rename View";
        renameModalValue = view.name;
        renameModalCallback = async (newName) => {
          if (newName && newName !== view.name) {
            console.log("Rename View:", view.name, "to", newName);
            // TODO: Implement rename view
          }
        };
        showRenameModal = true;
        break;
      case "refresh":
        // Refresh view list
        toggleDatabase(connection.id, database);
        break;
    }
    closeViewContextMenu();
  }
</script>

<div class="sidebar-container d-flex flex-column h-100">
  <div class="sidebar-header border-bottom">
    <div class="d-flex align-items-center justify-content-between mb-1">
      <h6
        class="text-uppercase mb-0 sidebar-title"
        style="font-size: 10px; font-weight: 600; letter-spacing: 0.5px;"
      >
        <i class="fas fa-network-wired me-1"></i>
        Connections
      </h6>
    </div>
    <div class="d-flex gap-2">
      <input
        type="search"
        class="form-control form-control-sm flex-grow-1"
        placeholder="Search connection or database"
        bind:value={searchQuery}
        style="font-size: 11px; height: 22px; padding: 2px 8px;"
      />
      <button
        class="btn btn-sm btn-success"
        on:click={openNewConnectionModal}
        style="font-size: 11px; padding: 2px 6px; height: 22px;"
        title="Add Connection"
      >
        <i class="fas fa-plus"></i>
      </button>
    </div>
  </div>

  <div class="sidebar-content flex-grow-1 overflow-auto p-1">
    {#if filteredConnections.length === 0}
      <p class="text-muted small p-3">No connections found</p>
    {/if}

    {#each filteredConnections as conn (conn.id)}
      <div class="tree-item">
        <div class="tree-node">
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
            class:active={activeContextConnection === conn.id}
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
                      class:table-active={$selectedTable?.name === cache.name &&
                        $selectedTable?._connId === conn.id}
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
                              size: null,
                            };
                            selectTable(table, conn.id, cache.name);
                          }}
                          on:dblclick={() => {
                            const table = {
                              name: cache.name,
                              schema: null,
                              size: null,
                            };
                            handleTableDoubleClick(table, conn, {
                              name: cache.name,
                            });
                          }}
                          on:contextmenu={(e) => {
                            const table = {
                              name: cache.name,
                              schema: null,
                              size: null,
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
                      class:active={activeContextDatabase ===
                        `${conn.id}-${db.name}`}
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
                                >Schemas ({expandedDatabases[
                                  `${conn.id}-${db.name}`
                                ]?.schemas?.length ?? 0})</span
                              >
                            </button>
                          </div>
                          {#if expandedSchemasParent[`${conn.id}-${db.name}`]?.schemas}
                            <div class="tree-children">
                              {#each expandedSchemasParent[`${conn.id}-${db.name}`].schemas as schema}
                                {@const schemaName = schema.name}
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
                                      {#if loadingSchemas[`${conn.id}-${db.name}-${schemaName}`]}
                                        <i class="fas fa-spinner fa-spin"></i>
                                      {:else}
                                        <i
                                          class="fas fa-chevron-{expandedSchemas[
                                            `${conn.id}-${db.name}-${schemaName}`
                                          ]
                                            ? 'down'
                                            : 'right'}"
                                        ></i>
                                      {/if}
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
                                          db
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
                                              toggleSchemaTables(
                                                conn.id,
                                                db.name,
                                                schemaName
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
                                              toggleSchemaTables(
                                                conn.id,
                                                db.name,
                                                schemaName
                                              )}
                                          >
                                            <i class="fas fa-table"></i>
                                            <span
                                              >Tables ({schemaObjectCounts[
                                                `${conn.id}-${db.name}-${schemaName}`
                                              ]?.tables ?? 0})</span
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
                                                {#each expandedTables[`${conn.id}-${db.name}-${schemaName}`]?.tables || [] as table (`${conn.id}-${db.name}-${schemaName}-${table.name}`)}
                                                  <tr
                                                    class="table-item-row"
                                                    class:table-active={($selectedTable?.name ===
                                                      table.name &&
                                                      $selectedTable?._connId ===
                                                        conn.id &&
                                                      $selectedTable?._dbName ===
                                                        db.name &&
                                                      $selectedTable?._schema ===
                                                        schemaName) ||
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
                                                            db.name,
                                                            schemaName
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
                                                      {#if table.size !== undefined && table.size !== null}
                                                        <span
                                                          class="badge bg-light text-secondary"
                                                          style="font-size: 10px;"
                                                          title="Table size"
                                                          >{table.size}</span
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

                                      <!-- PostgreSQL Schema Objects: Views, Indexes, Procedures, Triggers -->
                                      <!-- Views -->
                                      <div class="tree-item">
                                        <div
                                          class="tree-node tables-section-node"
                                        >
                                          <button
                                            class="tree-toggle"
                                            aria-label="Toggle views"
                                            on:click={() =>
                                              toggleSchemaViews(
                                                conn.id,
                                                db.name,
                                                schemaName
                                              )}
                                          >
                                            {#if loadingViews[`${conn.id}-${db.name}-${schemaName}`]}
                                              <i class="fas fa-spinner fa-spin"
                                              ></i>
                                            {:else}
                                              <i
                                                class="fas fa-chevron-{expandedViews[
                                                  `${conn.id}-${db.name}-${schemaName}`
                                                ]
                                                  ? 'down'
                                                  : 'right'}"
                                              ></i>
                                            {/if}
                                          </button>
                                          <button
                                            class="tree-section-header"
                                            on:click={() =>
                                              toggleSchemaViews(
                                                conn.id,
                                                db.name,
                                                schemaName
                                              )}
                                          >
                                            <i class="fas fa-eye"></i>
                                            <span
                                              >Views ({schemaObjectCounts[
                                                `${conn.id}-${db.name}-${schemaName}`
                                              ]?.views ?? 0})</span
                                            >
                                          </button>
                                        </div>
                                        {#if expandedViews[`${conn.id}-${db.name}-${schemaName}`]}
                                          <div class="tree-children">
                                            <table
                                              class="table table-sm table-hover mb-0 table-borderless mysql-object-table"
                                              style="padding-left: 8px;"
                                            >
                                              <tbody>
                                                {#each expandedViews[`${conn.id}-${db.name}-${schemaName}`]?.views || [] as view (`${schemaName}-${view.name}`)}
                                                  <tr
                                                    class="table-item-row"
                                                    class:table-active={activeContextView ===
                                                      `${conn.id}-${db.name}-${schemaName}-${view.name}`}
                                                    style="cursor: pointer; line-height: 1.5;"
                                                  >
                                                    <td
                                                      class="p-0 align-middle"
                                                      style="width: 100%; max-width: 0; overflow: hidden; white-space: nowrap; padding-left: 24px !important;"
                                                    >
                                                      <button
                                                        class="btn btn-sm p-1 text-start border-0 mysql-object-btn"
                                                        style="font-size: 11px; display: inline-block; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
                                                        on:click={() => {
                                                          const viewWithSchema =
                                                            {
                                                              ...view,
                                                              schema:
                                                                schemaName,
                                                            };
                                                          console.log(
                                                            "View clicked:",
                                                            viewWithSchema
                                                          );
                                                        }}
                                                        on:dblclick={() =>
                                                          handleViewDoubleClick(
                                                            {
                                                              ...view,
                                                              schema:
                                                                schemaName,
                                                            },
                                                            expandedDatabases[
                                                              `${conn.id}-${db.name}`
                                                            ].connection,
                                                            expandedDatabases[
                                                              `${conn.id}-${db.name}`
                                                            ].database
                                                          )}
                                                        on:contextmenu={(e) =>
                                                          handleViewContextMenu(
                                                            e,
                                                            {
                                                              ...view,
                                                              schema:
                                                                schemaName,
                                                            },
                                                            expandedDatabases[
                                                              `${conn.id}-${db.name}`
                                                            ].connection,
                                                            expandedDatabases[
                                                              `${conn.id}-${db.name}`
                                                            ].database
                                                          )}
                                                      >
                                                        <i
                                                          class="fas fa-eye mysql-object-icon me-1"
                                                          style="font-size: 11px;"
                                                        ></i>
                                                        <span
                                                          class="text-truncate"
                                                          title={view.name}
                                                          >{view.name}</span
                                                        >
                                                      </button>
                                                    </td>
                                                  </tr>
                                                {/each}
                                                {#if (expandedViews[`${conn.id}-${db.name}-${schemaName}`]?.views || []).length === 0}
                                                  <tr>
                                                    <td
                                                      class="p-0"
                                                      style="padding-left: 24px !important;"
                                                    >
                                                      <span
                                                        class="text-muted"
                                                        style="font-size: 11px; font-style: italic;"
                                                        >No views</span
                                                      >
                                                    </td>
                                                  </tr>
                                                {/if}
                                              </tbody>
                                            </table>
                                          </div>
                                        {/if}
                                      </div>

                                      <!-- Indexes -->
                                      <div class="tree-item">
                                        <div
                                          class="tree-node tables-section-node"
                                        >
                                          <button
                                            class="tree-toggle"
                                            aria-label="Toggle indexes"
                                            on:click={() =>
                                              toggleSchemaIndexes(
                                                conn.id,
                                                db.name,
                                                schemaName
                                              )}
                                          >
                                            {#if loadingIndexes[`${conn.id}-${db.name}-${schemaName}`]}
                                              <i class="fas fa-spinner fa-spin"
                                              ></i>
                                            {:else}
                                              <i
                                                class="fas fa-chevron-{expandedIndexes[
                                                  `${conn.id}-${db.name}-${schemaName}`
                                                ]
                                                  ? 'down'
                                                  : 'right'}"
                                              ></i>
                                            {/if}
                                          </button>
                                          <button
                                            class="tree-section-header"
                                            on:click={() =>
                                              toggleSchemaIndexes(
                                                conn.id,
                                                db.name,
                                                schemaName
                                              )}
                                          >
                                            <i class="fas fa-key"></i>
                                            <span
                                              >Indexes ({schemaObjectCounts[
                                                `${conn.id}-${db.name}-${schemaName}`
                                              ]?.indexes ?? 0})</span
                                            >
                                          </button>
                                        </div>
                                        {#if expandedIndexes[`${conn.id}-${db.name}-${schemaName}`]}
                                          <div class="tree-children">
                                            <table
                                              class="table table-sm table-hover mb-0 table-borderless mysql-object-table"
                                              style="padding-left: 8px;"
                                            >
                                              <tbody>
                                                {#each expandedIndexes[`${conn.id}-${db.name}-${schemaName}`]?.indexes || [] as idx (`${schemaName}-${idx.table_name}-${idx.name}`)}
                                                  <tr
                                                    class="table-item-row"
                                                    style="cursor: pointer; line-height: 1.5;"
                                                  >
                                                    <td
                                                      class="p-0 align-middle"
                                                      style="width: 100%; max-width: 0; overflow: hidden; white-space: nowrap; padding-left: 24px !important;"
                                                    >
                                                      <button
                                                        class="btn btn-sm p-1 text-start border-0 mysql-object-btn"
                                                        style="font-size: 11px; display: inline-block;  overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
                                                      >
                                                        <i
                                                          class="fas fa-key mysql-object-icon me-1"
                                                          style="font-size: 11px;"
                                                        ></i>
                                                        <span
                                                          class="text-truncate"
                                                          title="{idx.table_name}.{idx.name}"
                                                          >{idx.table_name}.{idx.name}</span
                                                        >
                                                      </button>
                                                    </td>
                                                    <td
                                                      class="text-end align-middle"
                                                      style="white-space: nowrap; width: 24px; min-width: 24px; max-width: 24px; padding: 2px 8px 2px 4px !important;"
                                                    >
                                                      {#if idx.is_unique}
                                                        <span
                                                          class="badge bg-info"
                                                          style="font-size: 9px; width: 16px; text-align: center;"
                                                          title="Unique Index"
                                                          >U</span
                                                        >
                                                      {:else}
                                                        <span
                                                          class="badge bg-secondary"
                                                          style="font-size: 9px; width: 16px; text-align: center;"
                                                          title="Index">I</span
                                                        >
                                                      {/if}
                                                    </td>
                                                  </tr>
                                                {/each}
                                                {#if (expandedIndexes[`${conn.id}-${db.name}-${schemaName}`]?.indexes || []).length === 0}
                                                  <tr>
                                                    <td
                                                      class="p-0"
                                                      colspan="2"
                                                      style="padding-left: 24px !important;"
                                                    >
                                                      <span
                                                        class="text-muted"
                                                        style="font-size: 11px; font-style: italic;"
                                                        >No indexes</span
                                                      >
                                                    </td>
                                                  </tr>
                                                {/if}
                                              </tbody>
                                            </table>
                                          </div>
                                        {/if}
                                      </div>

                                      <!-- Functions -->
                                      <div class="tree-item">
                                        <div
                                          class="tree-node tables-section-node"
                                        >
                                          <button
                                            class="tree-toggle"
                                            aria-label="Toggle functions"
                                            on:click={() =>
                                              toggleSchemaProcedures(
                                                conn.id,
                                                db.name,
                                                schemaName
                                              )}
                                          >
                                            {#if loadingProcedures[`${conn.id}-${db.name}-${schemaName}`]}
                                              <i class="fas fa-spinner fa-spin"
                                              ></i>
                                            {:else}
                                              <i
                                                class="fas fa-chevron-{expandedProcedures[
                                                  `${conn.id}-${db.name}-${schemaName}`
                                                ]
                                                  ? 'down'
                                                  : 'right'}"
                                              ></i>
                                            {/if}
                                          </button>
                                          <button
                                            class="tree-section-header"
                                            on:click={() =>
                                              toggleSchemaProcedures(
                                                conn.id,
                                                db.name,
                                                schemaName
                                              )}
                                          >
                                            <i class="fas fa-cog"></i>
                                            <span
                                              >Functions ({cachedSchemaProcedures[
                                                `${conn.id}-${db.name}-${schemaName}`
                                              ]?.length ?? 0})</span
                                            >
                                          </button>
                                        </div>
                                        {#if expandedProcedures[`${conn.id}-${db.name}-${schemaName}`]}
                                          <div class="tree-children">
                                            <table
                                              class="table table-sm table-hover mb-0 table-borderless mysql-object-table"
                                              style="padding-left: 8px;"
                                            >
                                              <tbody>
                                                {#each expandedProcedures[`${conn.id}-${db.name}-${schemaName}`]?.procedures || [] as proc (proc.oid || `${schemaName}-${proc.name}`)}
                                                  <tr
                                                    class="table-item-row"
                                                    style="cursor: pointer; line-height: 1.5;"
                                                  >
                                                    <td
                                                      class="p-0 align-middle"
                                                      style="width: 100%; max-width: 0; overflow: hidden; white-space: nowrap; padding-left: 24px !important;"
                                                    >
                                                      <button
                                                        class="btn btn-sm p-1 text-start border-0 mysql-object-btn"
                                                        style="font-size: 11px; display: inline-block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
                                                        on:dblclick={() =>
                                                          handleProcedureDoubleClick(
                                                            proc,
                                                            expandedDatabases[
                                                              `${conn.id}-${db.name}`
                                                            ].connection,
                                                            expandedDatabases[
                                                              `${conn.id}-${db.name}`
                                                            ].database,
                                                            schemaName
                                                          )}
                                                      >
                                                        <i
                                                          class="fas fa-cog mysql-object-icon me-1"
                                                          style="font-size: 11px;"
                                                        ></i>
                                                        <span
                                                          class="text-truncate"
                                                          title={proc.name}
                                                          >{proc.name}</span
                                                        >
                                                      </button>
                                                    </td>
                                                    <td
                                                      class="text-end align-middle"
                                                      style="white-space: nowrap; width: 24px; min-width: 24px; max-width: 24px; padding: 2px 8px 2px 4px !important;"
                                                    >
                                                      {#if proc.procedure_type === "FUNCTION"}
                                                        <span
                                                          class="badge bg-success"
                                                          style="font-size: 9px; width: 16px; text-align: center;"
                                                          title="Function"
                                                          >F</span
                                                        >
                                                      {:else if proc.procedure_type === "PROCEDURE"}
                                                        <span
                                                          class="badge bg-secondary"
                                                          style="font-size: 9px; width: 16px; text-align: center;"
                                                          title="Procedure"
                                                          >P</span
                                                        >
                                                      {/if}
                                                    </td>
                                                  </tr>
                                                {/each}
                                                {#if (expandedProcedures[`${conn.id}-${db.name}-${schemaName}`]?.procedures || []).length === 0}
                                                  <tr>
                                                    <td
                                                      class="p-0"
                                                      colspan="2"
                                                      style="padding-left: 24px !important;"
                                                    >
                                                      <span
                                                        class="text-muted"
                                                        style="font-size: 11px; font-style: italic;"
                                                        >No functions</span
                                                      >
                                                    </td>
                                                  </tr>
                                                {/if}
                                              </tbody>
                                            </table>
                                          </div>
                                        {/if}
                                      </div>

                                      <!-- Triggers -->
                                      <div class="tree-item">
                                        <div
                                          class="tree-node tables-section-node"
                                        >
                                          <button
                                            class="tree-toggle"
                                            aria-label="Toggle triggers"
                                            on:click={() =>
                                              toggleSchemaTriggers(
                                                conn.id,
                                                db.name,
                                                schemaName
                                              )}
                                          >
                                            {#if loadingTriggers[`${conn.id}-${db.name}-${schemaName}`]}
                                              <i class="fas fa-spinner fa-spin"
                                              ></i>
                                            {:else}
                                              <i
                                                class="fas fa-chevron-{expandedTriggers[
                                                  `${conn.id}-${db.name}-${schemaName}`
                                                ]
                                                  ? 'down'
                                                  : 'right'}"
                                              ></i>
                                            {/if}
                                          </button>
                                          <button
                                            class="tree-section-header"
                                            on:click={() =>
                                              toggleSchemaTriggers(
                                                conn.id,
                                                db.name,
                                                schemaName
                                              )}
                                          >
                                            <i class="fas fa-bolt"></i>
                                            <span
                                              >Triggers ({schemaObjectCounts[
                                                `${conn.id}-${db.name}-${schemaName}`
                                              ]?.triggers ?? 0})</span
                                            >
                                          </button>
                                        </div>
                                        {#if expandedTriggers[`${conn.id}-${db.name}-${schemaName}`]}
                                          <div class="tree-children">
                                            <table
                                              class="table table-sm table-hover mb-0 table-borderless mysql-object-table"
                                              style="padding-left: 8px;"
                                            >
                                              <tbody>
                                                {#each expandedTriggers[`${conn.id}-${db.name}-${schemaName}`]?.triggers || [] as trigger (`${schemaName}-${trigger.table_name}-${trigger.name}`)}
                                                  <tr
                                                    class="table-item-row"
                                                    style="cursor: pointer; line-height: 1.5;"
                                                  >
                                                    <td
                                                      class="p-0 align-middle"
                                                      style="width: 100%; max-width: 0; overflow: hidden; white-space: nowrap; padding-left: 24px !important;"
                                                    >
                                                      <button
                                                        class="btn btn-sm p-1 text-start border-0 mysql-object-btn"
                                                        style="font-size: 11px; display: inline-block; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
                                                      >
                                                        <i
                                                          class="fas fa-bolt mysql-object-icon me-1"
                                                          style="font-size: 11px;"
                                                        ></i>
                                                        <span
                                                          class="text-truncate"
                                                          title="{trigger.timing} {trigger.event} ON {trigger.table_name}"
                                                          >{trigger.name}</span
                                                        >
                                                      </button>
                                                    </td>
                                                  </tr>
                                                {/each}
                                                {#if (expandedTriggers[`${conn.id}-${db.name}-${schemaName}`]?.triggers || []).length === 0}
                                                  <tr>
                                                    <td
                                                      class="p-0"
                                                      style="padding-left: 24px !important;"
                                                    >
                                                      <span
                                                        class="text-muted"
                                                        style="font-size: 11px; font-style: italic;"
                                                        >No triggers</span
                                                      >
                                                    </td>
                                                  </tr>
                                                {/if}
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
                                {#if loadingSchemas[`${conn.id}-${db.name}-${schemaName}`]}
                                  <i class="fas fa-spinner fa-spin"></i>
                                {:else}
                                  <i
                                    class="fas fa-chevron-{expandedSchemas[
                                      `${conn.id}-${db.name}-${schemaName}`
                                    ]
                                      ? 'down'
                                      : 'right'}"
                                  ></i>
                                {/if}
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
                                      <span
                                        >Tables ({schemaObjectCounts[
                                          `${conn.id}-${db.name}-${schemaName}`
                                        ]?.tables ?? 0})</span
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
                                          {#each schemaTables as table (`${conn.id}-${db.name}-${schemaName}-${table.name}`)}
                                            <tr
                                              class="table-item-row"
                                              class:table-active={($selectedTable?.name ===
                                                table.name &&
                                                $selectedTable?._connId ===
                                                  conn.id &&
                                                $selectedTable?._dbName ===
                                                  db.name &&
                                                $selectedTable?._schema ===
                                                  schemaName) ||
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
                                                      db.name,
                                                      schemaName
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
                                                {#if table.size !== undefined && table.size !== null}
                                                  <span
                                                    class="badge bg-light text-secondary"
                                                    style="font-size: 10px;"
                                                    title="Table size"
                                                    >{table.size}</span
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

                                <!-- MSSQL Schema Objects: Views, Indexes, Procedures, Triggers -->
                                <!-- Views -->
                                <div class="tree-item">
                                  <div class="tree-node tables-section-node">
                                    <button
                                      class="tree-toggle"
                                      aria-label="Toggle views"
                                      on:click={() =>
                                        toggleSchemaViews(
                                          conn.id,
                                          db.name,
                                          schemaName
                                        )}
                                    >
                                      {#if loadingViews[`${conn.id}-${db.name}-${schemaName}`]}
                                        <i class="fas fa-spinner fa-spin"></i>
                                      {:else}
                                        <i
                                          class="fas fa-chevron-{expandedViews[
                                            `${conn.id}-${db.name}-${schemaName}`
                                          ]
                                            ? 'down'
                                            : 'right'}"
                                        ></i>
                                      {/if}
                                    </button>
                                    <button
                                      class="tree-section-header"
                                      on:click={() =>
                                        toggleSchemaViews(
                                          conn.id,
                                          db.name,
                                          schemaName
                                        )}
                                    >
                                      <i class="fas fa-eye"></i>
                                      <span
                                        >Views ({schemaObjectCounts[
                                          `${conn.id}-${db.name}-${schemaName}`
                                        ]?.views ?? 0})</span
                                      >
                                    </button>
                                  </div>
                                  {#if expandedViews[`${conn.id}-${db.name}-${schemaName}`]}
                                    <div class="tree-children">
                                      <table
                                        class="table table-sm table-hover mb-0 table-borderless mysql-object-table"
                                        style="padding-left: 8px;"
                                      >
                                        <tbody>
                                          {#each expandedViews[`${conn.id}-${db.name}-${schemaName}`]?.views || [] as view (`${schemaName}-${view.name}`)}
                                            <tr
                                              class="table-item-row"
                                              class:table-active={activeContextView ===
                                                `${conn.id}-${db.name}-${schemaName}-${view.name}`}
                                              style="cursor: pointer; line-height: 1.5;"
                                            >
                                              <td
                                                class="p-0 align-middle"
                                                style="width: 100%; max-width: 0; overflow: hidden; white-space: nowrap; padding-left: 24px !important;"
                                              >
                                                <button
                                                  class="btn btn-sm p-1 text-start border-0 mysql-object-btn"
                                                  style="font-size: 11px; display: inline-block; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
                                                  on:click={() => {
                                                    const viewWithSchema = {
                                                      ...view,
                                                      schema: schemaName,
                                                    };
                                                    console.log(
                                                      "View clicked:",
                                                      viewWithSchema
                                                    );
                                                  }}
                                                  on:dblclick={() =>
                                                    handleViewDoubleClick(
                                                      {
                                                        ...view,
                                                        schema: schemaName,
                                                      },
                                                      expandedDatabases[
                                                        `${conn.id}-${db.name}`
                                                      ].connection,
                                                      expandedDatabases[
                                                        `${conn.id}-${db.name}`
                                                      ].database
                                                    )}
                                                  on:contextmenu={(e) =>
                                                    handleViewContextMenu(
                                                      e,
                                                      {
                                                        ...view,
                                                        schema: schemaName,
                                                      },
                                                      expandedDatabases[
                                                        `${conn.id}-${db.name}`
                                                      ].connection,
                                                      expandedDatabases[
                                                        `${conn.id}-${db.name}`
                                                      ].database
                                                    )}
                                                >
                                                  <i
                                                    class="fas fa-eye mysql-object-icon me-1"
                                                    style="font-size: 11px;"
                                                  ></i>
                                                  <span
                                                    class="text-truncate"
                                                    title={view.name}
                                                    >{view.name}</span
                                                  >
                                                </button>
                                              </td>
                                            </tr>
                                          {/each}
                                          {#if (expandedViews[`${conn.id}-${db.name}-${schemaName}`]?.views || []).length === 0}
                                            <tr>
                                              <td
                                                class="p-0"
                                                style="padding-left: 24px !important;"
                                              >
                                                <span
                                                  class="text-muted"
                                                  style="font-size: 11px; font-style: italic;"
                                                  >No views</span
                                                >
                                              </td>
                                            </tr>
                                          {/if}
                                        </tbody>
                                      </table>
                                    </div>
                                  {/if}
                                </div>

                                <!-- Indexes -->
                                <div class="tree-item">
                                  <div class="tree-node tables-section-node">
                                    <button
                                      class="tree-toggle"
                                      aria-label="Toggle indexes"
                                      on:click={() =>
                                        toggleSchemaIndexes(
                                          conn.id,
                                          db.name,
                                          schemaName
                                        )}
                                    >
                                      {#if loadingIndexes[`${conn.id}-${db.name}-${schemaName}`]}
                                        <i class="fas fa-spinner fa-spin"></i>
                                      {:else}
                                        <i
                                          class="fas fa-chevron-{expandedIndexes[
                                            `${conn.id}-${db.name}-${schemaName}`
                                          ]
                                            ? 'down'
                                            : 'right'}"
                                        ></i>
                                      {/if}
                                    </button>
                                    <button
                                      class="tree-section-header"
                                      on:click={() =>
                                        toggleSchemaIndexes(
                                          conn.id,
                                          db.name,
                                          schemaName
                                        )}
                                    >
                                      <i class="fas fa-key"></i>
                                      <span
                                        >Indexes ({schemaObjectCounts[
                                          `${conn.id}-${db.name}-${schemaName}`
                                        ]?.indexes ?? 0})</span
                                      >
                                    </button>
                                  </div>
                                  {#if expandedIndexes[`${conn.id}-${db.name}-${schemaName}`]}
                                    <div class="tree-children">
                                      <table
                                        class="table table-sm table-hover mb-0 table-borderless mysql-object-table"
                                        style="padding-left: 8px;"
                                      >
                                        <tbody>
                                          {#each expandedIndexes[`${conn.id}-${db.name}-${schemaName}`]?.indexes || [] as idx (`${schemaName}-${idx.table_name}-${idx.name}`)}
                                            <tr
                                              class="table-item-row"
                                              style="cursor: pointer; line-height: 1.5;"
                                            >
                                              <td
                                                class="p-0 align-middle"
                                                style="width: 100%; max-width: 0; overflow: hidden; white-space: nowrap; padding-left: 24px !important;"
                                              >
                                                <button
                                                  class="btn btn-sm p-1 text-start border-0 mysql-object-btn"
                                                  style="font-size: 11px; display: inline-block;  overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
                                                >
                                                  <i
                                                    class="fas fa-key mysql-object-icon me-1"
                                                    style="font-size: 11px;"
                                                  ></i>
                                                  <span
                                                    class="text-truncate"
                                                    title="{idx.table_name}.{idx.name}"
                                                    >{idx.table_name}.{idx.name}</span
                                                  >
                                                </button>
                                              </td>
                                              <td
                                                class="text-end align-middle"
                                                style="white-space: nowrap; width: 24px; min-width: 24px; max-width: 24px; padding: 2px 8px 2px 4px !important;"
                                              >
                                                {#if idx.is_unique}
                                                  <span
                                                    class="badge bg-info"
                                                    style="font-size: 9px; width: 16px; text-align: center;"
                                                    title="Unique Index">U</span
                                                  >
                                                {:else}
                                                  <span
                                                    class="badge bg-secondary"
                                                    style="font-size: 9px; width: 16px; text-align: center;"
                                                    title="Index">I</span
                                                  >
                                                {/if}
                                              </td>
                                            </tr>
                                          {/each}
                                          {#if (expandedIndexes[`${conn.id}-${db.name}-${schemaName}`]?.indexes || []).length === 0}
                                            <tr>
                                              <td
                                                class="p-0"
                                                colspan="2"
                                                style="padding-left: 24px !important;"
                                              >
                                                <span
                                                  class="text-muted"
                                                  style="font-size: 11px; font-style: italic;"
                                                  >No indexes</span
                                                >
                                              </td>
                                            </tr>
                                          {/if}
                                        </tbody>
                                      </table>
                                    </div>
                                  {/if}
                                </div>

                                <!-- Procedures -->
                                <div class="tree-item">
                                  <div class="tree-node tables-section-node">
                                    <button
                                      class="tree-toggle"
                                      aria-label="Toggle procedures"
                                      on:click={() =>
                                        toggleSchemaProcedures(
                                          conn.id,
                                          db.name,
                                          schemaName
                                        )}
                                    >
                                      {#if loadingProcedures[`${conn.id}-${db.name}-${schemaName}`]}
                                        <i class="fas fa-spinner fa-spin"></i>
                                      {:else}
                                        <i
                                          class="fas fa-chevron-{expandedProcedures[
                                            `${conn.id}-${db.name}-${schemaName}`
                                          ]
                                            ? 'down'
                                            : 'right'}"
                                        ></i>
                                      {/if}
                                    </button>
                                    <button
                                      class="tree-section-header"
                                      on:click={() =>
                                        toggleSchemaProcedures(
                                          conn.id,
                                          db.name,
                                          schemaName
                                        )}
                                    >
                                      <i class="fas fa-cog"></i>
                                      <span
                                        >Procedures ({schemaObjectCounts[
                                          `${conn.id}-${db.name}-${schemaName}`
                                        ]?.procedures ?? 0})</span
                                      >
                                    </button>
                                  </div>
                                  {#if expandedProcedures[`${conn.id}-${db.name}-${schemaName}`]}
                                    <div class="tree-children">
                                      <table
                                        class="table table-sm table-hover mb-0 table-borderless mysql-object-table"
                                        style="padding-left: 8px;"
                                      >
                                        <tbody>
                                          {#each expandedProcedures[`${conn.id}-${db.name}-${schemaName}`]?.procedures || [] as proc (proc.oid || `${schemaName}-${proc.name}`)}
                                            <tr
                                              class="table-item-row"
                                              style="cursor: pointer; line-height: 1.5;"
                                            >
                                              <td
                                                class="p-0 align-middle"
                                                style="width: 100%; max-width: 0; overflow: hidden; white-space: nowrap; padding-left: 24px !important;"
                                              >
                                                <button
                                                  class="btn btn-sm p-1 text-start border-0 mysql-object-btn"
                                                  style="font-size: 11px; display: inline-block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
                                                  on:dblclick={() =>
                                                    handleProcedureDoubleClick(
                                                      proc,
                                                      expandedDatabases[
                                                        `${conn.id}-${db.name}`
                                                      ].connection,
                                                      expandedDatabases[
                                                        `${conn.id}-${db.name}`
                                                      ].database,
                                                      schemaName
                                                    )}
                                                >
                                                  <i
                                                    class="fas fa-cog mysql-object-icon me-1"
                                                    style="font-size: 11px;"
                                                  ></i>
                                                  <span
                                                    class="text-truncate"
                                                    title={proc.name}
                                                    >{proc.name}</span
                                                  >
                                                </button>
                                              </td>
                                              <td
                                                class="text-end align-middle"
                                                style="white-space: nowrap; width: 24px; min-width: 24px; max-width: 24px; padding: 2px 8px 2px 4px !important;"
                                              >
                                                {#if proc.procedure_type === "FUNCTION"}
                                                  <span
                                                    class="badge bg-success"
                                                    style="font-size: 9px; width: 16px; text-align: center;"
                                                    title="Function">F</span
                                                  >
                                                {:else if proc.procedure_type === "PROCEDURE"}
                                                  <span
                                                    class="badge bg-secondary"
                                                    style="font-size: 9px; width: 16px; text-align: center;"
                                                    title="Procedure">P</span
                                                  >
                                                {/if}
                                              </td>
                                            </tr>
                                          {/each}
                                          {#if (expandedProcedures[`${conn.id}-${db.name}-${schemaName}`]?.procedures || []).length === 0}
                                            <tr>
                                              <td
                                                class="p-0"
                                                colspan="2"
                                                style="padding-left: 24px !important;"
                                              >
                                                <span
                                                  class="text-muted"
                                                  style="font-size: 11px; font-style: italic;"
                                                  >No procedures</span
                                                >
                                              </td>
                                            </tr>
                                          {/if}
                                        </tbody>
                                      </table>
                                    </div>
                                  {/if}
                                </div>

                                <!-- Triggers -->
                                <div class="tree-item">
                                  <div class="tree-node tables-section-node">
                                    <button
                                      class="tree-toggle"
                                      aria-label="Toggle triggers"
                                      on:click={() =>
                                        toggleSchemaTriggers(
                                          conn.id,
                                          db.name,
                                          schemaName
                                        )}
                                    >
                                      {#if loadingTriggers[`${conn.id}-${db.name}-${schemaName}`]}
                                        <i class="fas fa-spinner fa-spin"></i>
                                      {:else}
                                        <i
                                          class="fas fa-chevron-{expandedTriggers[
                                            `${conn.id}-${db.name}-${schemaName}`
                                          ]
                                            ? 'down'
                                            : 'right'}"
                                        ></i>
                                      {/if}
                                    </button>
                                    <button
                                      class="tree-section-header"
                                      on:click={() =>
                                        toggleSchemaTriggers(
                                          conn.id,
                                          db.name,
                                          schemaName
                                        )}
                                    >
                                      <i class="fas fa-bolt"></i>
                                      <span
                                        >Triggers ({schemaObjectCounts[
                                          `${conn.id}-${db.name}-${schemaName}`
                                        ]?.triggers ?? 0})</span
                                      >
                                    </button>
                                  </div>
                                  {#if expandedTriggers[`${conn.id}-${db.name}-${schemaName}`]}
                                    <div class="tree-children">
                                      <table
                                        class="table table-sm table-hover mb-0 table-borderless mysql-object-table"
                                        style="padding-left: 8px;"
                                      >
                                        <tbody>
                                          {#each expandedTriggers[`${conn.id}-${db.name}-${schemaName}`]?.triggers || [] as trigger (`${schemaName}-${trigger.table_name}-${trigger.name}`)}
                                            <tr
                                              class="table-item-row"
                                              style="cursor: pointer; line-height: 1.5;"
                                            >
                                              <td
                                                class="p-0 align-middle"
                                                style="width: 100%; max-width: 0; overflow: hidden; white-space: nowrap; padding-left: 24px !important;"
                                              >
                                                <button
                                                  class="btn btn-sm p-1 text-start border-0 mysql-object-btn"
                                                  style="font-size: 11px; display: inline-block; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
                                                >
                                                  <i
                                                    class="fas fa-bolt mysql-object-icon me-1"
                                                    style="font-size: 11px;"
                                                  ></i>
                                                  <span
                                                    class="text-truncate"
                                                    title="{trigger.timing} {trigger.event} ON {trigger.table_name}"
                                                    >{trigger.name}</span
                                                  >
                                                </button>
                                              </td>
                                            </tr>
                                          {/each}
                                          {#if (expandedTriggers[`${conn.id}-${db.name}-${schemaName}`]?.triggers || []).length === 0}
                                            <tr>
                                              <td
                                                class="p-0"
                                                style="padding-left: 24px !important;"
                                              >
                                                <span
                                                  class="text-muted"
                                                  style="font-size: 11px; font-style: italic;"
                                                  >No triggers</span
                                                >
                                              </td>
                                            </tr>
                                          {/if}
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
                                    : "Tables"} ({dbObjectCounts[
                                  `${conn.id}-${db.name}`
                                ].tables ?? 0})</span
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
                                  {#each expandedTables[`${conn.id}-${db.name}`]?.tables || [] as table (`${conn.id}-${db.name}-${table.schema || "default"}-${table.name}`)}
                                    <tr
                                      class="table-item-row"
                                      class:table-active={($selectedTable?.name ===
                                        table.name &&
                                        $selectedTable?._connId === conn.id &&
                                        $selectedTable?._dbName === db.name &&
                                        ($selectedTable?._schema ===
                                          table.schema ||
                                          $selectedTable?._schema ===
                                            (table.schema || null))) ||
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
                                              db.name,
                                              table.schema
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
                                        {#if table.size !== undefined && table.size !== null}
                                          <span
                                            class="badge bg-light text-secondary"
                                            style="font-size: 10px;"
                                            title="{conn.db_type === 'MongoDB'
                                              ? 'Collection'
                                              : conn.db_type === 'Ignite'
                                                ? 'Cache'
                                                : 'Table'} size"
                                            >{table.size}</span
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

                        {#if conn.db_type === "MySQL"}
                          <!-- MySQL specific objects: Views, Indexes, Procedures, Triggers, Events -->
                          <!-- Views -->
                          <div class="tree-item">
                            <div class="tree-node tables-section-node">
                              <button
                                class="tree-toggle"
                                aria-label="Toggle views"
                                on:click={() =>
                                  toggleViews(conn.id, db.name, conn)}
                              >
                                {#if loadingViews[`${conn.id}-${db.name}`]}
                                  <i class="fas fa-spinner fa-spin"></i>
                                {:else}
                                  <i
                                    class="fas fa-chevron-{expandedViews[
                                      `${conn.id}-${db.name}`
                                    ]
                                      ? 'down'
                                      : 'right'}"
                                  ></i>
                                {/if}
                              </button>
                              <button
                                class="tree-section-header"
                                on:click={() =>
                                  toggleViews(conn.id, db.name, conn)}
                              >
                                <i class="fas fa-eye"></i>
                                <span
                                  >Views ({dbObjectCounts[
                                    `${conn.id}-${db.name}`
                                  ]?.views ?? 0})</span
                                >
                              </button>
                            </div>
                            {#if expandedViews[`${conn.id}-${db.name}`]}
                              <div class="tree-children">
                                <table
                                  class="table table-sm table-hover mb-0 table-borderless mysql-object-table"
                                  style="padding-left: 8px;"
                                >
                                  <tbody>
                                    {#each expandedViews[`${conn.id}-${db.name}`]?.views || [] as view (view.name)}
                                      <tr
                                        class="table-item-row"
                                        class:table-active={activeContextView ===
                                          `${conn.id}-${db.name}-${view.schema || "public"}-${view.name}`}
                                        style="cursor: pointer; line-height: 1.5;"
                                      >
                                        <td
                                          class="p-0 align-middle"
                                          style="width: 100%; max-width: 0; overflow: hidden; white-space: nowrap; padding-left: 24px !important;"
                                        >
                                          <button
                                            class="btn btn-sm p-1 text-start border-0 mysql-object-btn"
                                            style="font-size: 11px; display: inline-block; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
                                            on:click={() => {
                                              console.log(
                                                "View clicked:",
                                                view
                                              );
                                            }}
                                            on:dblclick={() =>
                                              handleViewDoubleClick(
                                                view,
                                                expandedDatabases[
                                                  `${conn.id}-${db.name}`
                                                ].connection,
                                                expandedDatabases[
                                                  `${conn.id}-${db.name}`
                                                ].database
                                              )}
                                            on:contextmenu={(e) =>
                                              handleViewContextMenu(
                                                e,
                                                view,
                                                expandedDatabases[
                                                  `${conn.id}-${db.name}`
                                                ].connection,
                                                expandedDatabases[
                                                  `${conn.id}-${db.name}`
                                                ].database
                                              )}
                                          >
                                            <i
                                              class="fas fa-eye mysql-object-icon me-1"
                                              style="font-size: 11px;"
                                            ></i>
                                            <span
                                              class="text-truncate"
                                              title={view.name}
                                              >{view.name}</span
                                            >
                                          </button>
                                        </td>
                                      </tr>
                                    {/each}
                                    {#if (expandedViews[`${conn.id}-${db.name}`]?.views || []).length === 0}
                                      <tr>
                                        <td
                                          class="p-0"
                                          style="padding-left: 24px !important;"
                                        >
                                          <span
                                            class="text-muted"
                                            style="font-size: 11px; font-style: italic;"
                                            >No views</span
                                          >
                                        </td>
                                      </tr>
                                    {/if}
                                  </tbody>
                                </table>
                              </div>
                            {/if}
                          </div>

                          <!-- Indexes -->
                          <div class="tree-item">
                            <div class="tree-node tables-section-node">
                              <button
                                class="tree-toggle"
                                aria-label="Toggle indexes"
                                on:click={() =>
                                  toggleIndexes(conn.id, db.name, conn)}
                              >
                                {#if loadingIndexes[`${conn.id}-${db.name}`]}
                                  <i class="fas fa-spinner fa-spin"></i>
                                {:else}
                                  <i
                                    class="fas fa-chevron-{expandedIndexes[
                                      `${conn.id}-${db.name}`
                                    ]
                                      ? 'down'
                                      : 'right'}"
                                  ></i>
                                {/if}
                              </button>
                              <button
                                class="tree-section-header"
                                on:click={() => {
                                  console.log(
                                    "Indexes count for",
                                    `${conn.id}-${db.name}`,
                                    ":",
                                    cachedIndexes[`${conn.id}-${db.name}`]
                                      ?.length
                                  );
                                  toggleIndexes(conn.id, db.name, conn);
                                }}
                              >
                                <i class="fas fa-key"></i>
                                <span
                                  >Indexes ({dbObjectCounts[
                                    `${conn.id}-${db.name}`
                                  ]?.indexes ?? 0})</span
                                >
                              </button>
                            </div>
                            {#if expandedIndexes[`${conn.id}-${db.name}`]}
                              <div class="tree-children">
                                <table
                                  class="table table-sm table-hover mb-0 table-borderless mysql-object-table"
                                  style="padding-left: 8px;"
                                >
                                  <tbody>
                                    {#each expandedIndexes[`${conn.id}-${db.name}`]?.indexes || [] as idx, i (`${conn.id}-${db.name}-${idx.table_name}-${idx.name}-${i}`)}
                                      <tr
                                        class="table-item-row"
                                        style="cursor: pointer; line-height: 1.5;"
                                      >
                                        <td
                                          class="p-0 align-middle"
                                          style="width: 100%; max-width: 0; overflow: hidden; white-space: nowrap; padding-left: 24px !important;"
                                        >
                                          <button
                                            class="btn btn-sm p-1 text-start border-0 mysql-object-btn"
                                            style="font-size: 11px; display: inline-block;  overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
                                          >
                                            <i
                                              class="fas fa-key mysql-object-icon me-1"
                                              style="font-size: 11px;"
                                            ></i>
                                            <span
                                              class="text-truncate"
                                              title="{idx.table_name}.{idx.name}"
                                              >{idx.table_name}.{idx.name}</span
                                            >
                                          </button>
                                        </td>
                                        <td
                                          class="text-end align-middle"
                                          style="white-space: nowrap; width: 24px; min-width: 24px; max-width: 24px; padding: 2px 8px 2px 4px !important;"
                                        >
                                          {#if idx.is_unique}
                                            <span
                                              class="badge bg-info"
                                              style="font-size: 9px; width: 16px; text-align: center;"
                                              title="Unique Index">U</span
                                            >
                                          {:else}
                                            <span
                                              class="badge bg-secondary"
                                              style="font-size: 9px; width: 16px; text-align: center;"
                                              title="Index">I</span
                                            >
                                          {/if}
                                        </td>
                                      </tr>
                                    {/each}
                                    {#if (expandedIndexes[`${conn.id}-${db.name}`]?.indexes || []).length === 0}
                                      <tr>
                                        <td
                                          class="p-0"
                                          colspan="2"
                                          style="padding-left: 24px !important;"
                                        >
                                          <span
                                            class="text-muted"
                                            style="font-size: 11px; font-style: italic;"
                                            >No indexes</span
                                          >
                                        </td>
                                      </tr>
                                    {/if}
                                  </tbody>
                                </table>
                              </div>
                            {/if}
                          </div>

                          <!-- Procedures -->
                          <div class="tree-item">
                            <div class="tree-node tables-section-node">
                              <button
                                class="tree-toggle"
                                aria-label="Toggle procedures"
                                on:click={() =>
                                  toggleProcedures(conn.id, db.name)}
                              >
                                {#if loadingProcedures[`${conn.id}-${db.name}`]}
                                  <i class="fas fa-spinner fa-spin"></i>
                                {:else}
                                  <i
                                    class="fas fa-chevron-{expandedProcedures[
                                      `${conn.id}-${db.name}`
                                    ]
                                      ? 'down'
                                      : 'right'}"
                                  ></i>
                                {/if}
                              </button>
                              <button
                                class="tree-section-header"
                                on:click={() => {
                                  console.log(
                                    "Procedures count for",
                                    `${conn.id}-${db.name}`,
                                    ":",
                                    dbObjectCounts[`${conn.id}-${db.name}`]
                                      ?.procedures
                                  );
                                  toggleProcedures(conn.id, db.name);
                                }}
                              >
                                <i class="fas fa-cog"></i>
                                <span
                                  >Procedures ({dbObjectCounts[
                                    `${conn.id}-${db.name}`
                                  ]?.procedures ?? 0})</span
                                >
                              </button>
                            </div>
                            {#if expandedProcedures[`${conn.id}-${db.name}`]}
                              <div class="tree-children">
                                <table
                                  class="table table-sm table-hover mb-0 table-borderless mysql-object-table"
                                  style="padding-left: 8px;"
                                >
                                  <tbody>
                                    {#each expandedProcedures[`${conn.id}-${db.name}`]?.procedures || [] as proc (proc.oid || proc.name)}
                                      <tr
                                        class="table-item-row"
                                        style="cursor: pointer; line-height: 1.5;"
                                      >
                                        <td
                                          class="p-0 align-middle"
                                          style="width: 100%; max-width: 0; overflow: hidden; white-space: nowrap; padding-left: 24px !important;"
                                        >
                                          <button
                                            class="btn btn-sm p-1 text-start border-0 mysql-object-btn"
                                            style="font-size: 11px; display: inline-block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
                                            on:dblclick={() =>
                                              handleProcedureDoubleClick(
                                                proc,
                                                expandedDatabases[
                                                  `${conn.id}-${db.name}`
                                                ].connection,
                                                expandedDatabases[
                                                  `${conn.id}-${db.name}`
                                                ].database
                                              )}
                                          >
                                            <i
                                              class="fas fa-cog mysql-object-icon me-1"
                                              style="font-size: 11px;"
                                            ></i>
                                            <span
                                              class="text-truncate"
                                              title={proc.name}
                                              >{proc.name}</span
                                            >
                                          </button>
                                        </td>
                                        <td
                                          class="text-end align-middle"
                                          style="white-space: nowrap; width: 24px; min-width: 24px; max-width: 24px; padding: 2px 8px 2px 4px !important;"
                                        >
                                          {#if proc.procedure_type === "FUNCTION"}
                                            <span
                                              class="badge bg-success"
                                              style="font-size: 9px; width: 16px; text-align: center;"
                                              title="Function">F</span
                                            >
                                          {:else if proc.procedure_type === "PROCEDURE"}
                                            <span
                                              class="badge bg-secondary"
                                              style="font-size: 9px; width: 16px; text-align: center;"
                                              title="Procedure">P</span
                                            >
                                          {/if}
                                        </td>
                                      </tr>
                                    {/each}
                                    {#if (expandedProcedures[`${conn.id}-${db.name}`]?.procedures || []).length === 0}
                                      <tr>
                                        <td
                                          class="p-0"
                                          colspan="2"
                                          style="padding-left: 24px !important;"
                                        >
                                          <span
                                            class="text-muted"
                                            style="font-size: 11px; font-style: italic;"
                                            >No procedures</span
                                          >
                                        </td>
                                      </tr>
                                    {/if}
                                  </tbody>
                                </table>
                              </div>
                            {/if}
                          </div>

                          <!-- Triggers -->
                          <div class="tree-item">
                            <div class="tree-node tables-section-node">
                              <button
                                class="tree-toggle"
                                aria-label="Toggle triggers"
                                on:click={() =>
                                  toggleTriggers(conn.id, db.name)}
                              >
                                {#if loadingTriggers[`${conn.id}-${db.name}`]}
                                  <i class="fas fa-spinner fa-spin"></i>
                                {:else}
                                  <i
                                    class="fas fa-chevron-{expandedTriggers[
                                      `${conn.id}-${db.name}`
                                    ]
                                      ? 'down'
                                      : 'right'}"
                                  ></i>
                                {/if}
                              </button>
                              <button
                                class="tree-section-header"
                                on:click={() =>
                                  toggleTriggers(conn.id, db.name)}
                              >
                                <i class="fas fa-bolt"></i>
                                <span
                                  >Triggers ({dbObjectCounts[
                                    `${conn.id}-${db.name}`
                                  ]?.triggers ?? 0})</span
                                >
                              </button>
                            </div>
                            {#if expandedTriggers[`${conn.id}-${db.name}`]}
                              <div class="tree-children">
                                <table
                                  class="table table-sm table-hover mb-0 table-borderless mysql-object-table"
                                  style="padding-left: 8px;"
                                >
                                  <tbody>
                                    {#each expandedTriggers[`${conn.id}-${db.name}`]?.triggers || [] as trigger (trigger.name)}
                                      <tr
                                        class="table-item-row"
                                        style="cursor: pointer; line-height: 1.5;"
                                      >
                                        <td
                                          class="p-0 align-middle"
                                          style="width: 100%; max-width: 0; overflow: hidden; white-space: nowrap; padding-left: 24px !important;"
                                        >
                                          <button
                                            class="btn btn-sm p-1 text-start border-0 mysql-object-btn"
                                            style="font-size: 11px; display: inline-block; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
                                          >
                                            <i
                                              class="fas fa-bolt mysql-object-icon me-1"
                                              style="font-size: 11px;"
                                            ></i>
                                            <span
                                              class="text-truncate"
                                              title="{trigger.timing} {trigger.event} ON {trigger.table_name}"
                                              >{trigger.name}</span
                                            >
                                          </button>
                                        </td>
                                      </tr>
                                    {/each}
                                    {#if (expandedTriggers[`${conn.id}-${db.name}`]?.triggers || []).length === 0}
                                      <tr>
                                        <td
                                          class="p-0"
                                          style="padding-left: 24px !important;"
                                        >
                                          <span
                                            class="text-muted"
                                            style="font-size: 11px; font-style: italic;"
                                            >No triggers</span
                                          >
                                        </td>
                                      </tr>
                                    {/if}
                                  </tbody>
                                </table>
                              </div>
                            {/if}
                          </div>

                          <!-- Events -->
                          <div class="tree-item">
                            <div class="tree-node tables-section-node">
                              <button
                                class="tree-toggle"
                                aria-label="Toggle events"
                                on:click={() => toggleEvents(conn.id, db.name)}
                              >
                                {#if loadingEvents[`${conn.id}-${db.name}`]}
                                  <i class="fas fa-spinner fa-spin"></i>
                                {:else}
                                  <i
                                    class="fas fa-chevron-{expandedEvents[
                                      `${conn.id}-${db.name}`
                                    ]
                                      ? 'down'
                                      : 'right'}"
                                  ></i>
                                {/if}
                              </button>
                              <button
                                class="tree-section-header"
                                on:click={() => toggleEvents(conn.id, db.name)}
                              >
                                <i class="fas fa-calendar-alt"></i>
                                <span
                                  >Events ({dbObjectCounts[
                                    `${conn.id}-${db.name}`
                                  ]?.events ?? 0})</span
                                >
                              </button>
                            </div>
                            {#if expandedEvents[`${conn.id}-${db.name}`]}
                              <div class="tree-children">
                                <table
                                  class="table table-sm table-hover mb-0 table-borderless mysql-object-table"
                                  style="padding-left: 8px;"
                                >
                                  <tbody>
                                    {#each expandedEvents[`${conn.id}-${db.name}`]?.events || [] as event (event.name)}
                                      <tr
                                        class="table-item-row"
                                        style="cursor: pointer; line-height: 1.5;"
                                      >
                                        <td
                                          class="p-0 align-middle"
                                          style="width: 100%; max-width: 0; overflow: hidden; white-space: nowrap; padding-left: 24px !important;"
                                        >
                                          <button
                                            class="btn btn-sm p-1 text-start border-0 mysql-object-btn"
                                            style="font-size: 11px; display: inline-block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
                                          >
                                            <i
                                              class="fas fa-calendar-alt mysql-object-icon me-1"
                                              style="font-size: 11px;"
                                            ></i>
                                            <span
                                              class="text-truncate"
                                              title={event.name}
                                              >{event.name}</span
                                            >
                                          </button>
                                        </td>
                                        <td
                                          class="text-end align-middle"
                                          style="white-space: nowrap; width: 24px; min-width: 24px; max-width: 24px; padding: 2px 8px 2px 4px !important;"
                                        >
                                          {#if event.status}
                                            <span
                                              class="badge {event.status ===
                                              'ENABLED'
                                                ? 'bg-success'
                                                : 'bg-warning'}"
                                              style="font-size: 9px; width: 16px; text-align: center;"
                                              title={event.status}
                                              >{event.status === "ENABLED"
                                                ? "E"
                                                : "D"}</span
                                            >
                                          {/if}
                                        </td>
                                      </tr>
                                    {/each}
                                    {#if (expandedEvents[`${conn.id}-${db.name}`]?.events || []).length === 0}
                                      <tr>
                                        <td
                                          class="p-0"
                                          colspan="2"
                                          style="padding-left: 24px !important;"
                                        >
                                          <span
                                            class="text-muted"
                                            style="font-size: 11px; font-style: italic;"
                                            >No events</span
                                          >
                                        </td>
                                      </tr>
                                    {/if}
                                  </tbody>
                                </table>
                              </div>
                            {/if}
                          </div>
                        {:else if conn.db_type !== "MongoDB" && conn.db_type !== "Redis" && conn.db_type !== "Ignite"}
                          <!-- Other SQL databases: Views and Functions -->
                          <div class="tree-item">
                            <div class="tree-node tables-section-node">
                              <button
                                class="tree-toggle"
                                aria-label="Toggle views"
                              >
                                <i class="fas fa-chevron-right"></i>
                              </button>
                              <button class="tree-section-header">
                                <i class="fas fa-eye"></i>
                                <span>Views</span>
                              </button>
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
                              <button class="tree-section-header">
                                <i class="fas fa-code"></i>
                                <span>Functions</span>
                              </button>
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

{#if viewContextMenu}
  {#key `${viewContextMenu.connection?.id}-${viewContextMenu.database?.name}-${viewContextMenu.view?.name}`}
    <ViewContextMenu
      x={viewContextMenu.x}
      y={viewContextMenu.y}
      view={viewContextMenu.view}
      connection={viewContextMenu.connection}
      database={viewContextMenu.database}
      on:viewStructure={handleViewAction}
      on:viewDefinition={handleViewAction}
      on:viewData={handleViewAction}
      on:exportData={handleViewAction}
      on:importData={handleViewAction}
      on:readInConsole={handleViewAction}
      on:copy={handleViewAction}
      on:copyAdvancedInfo={handleViewAction}
      on:delete={handleViewAction}
      on:rename={handleViewAction}
      on:refresh={handleViewAction}
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

<InputModal
  bind:show={showRenameModal}
  title={renameModalTitle}
  label="Enter new name:"
  value={renameModalValue}
  placeholder="Name"
  on:submit={handleRenameSubmit}
  on:cancel={() => (showRenameModal = false)}
/>

<style>
  /* Sidebar container styles */
  .sidebar-container {
    background: var(--bg-sidebar);
    color: var(--text-primary);
  }

  .sidebar-header {
    background: var(--bg-tertiary);
    height: 66px; /* tabbar (26px) + datagrid header (24px) + toolbar area */
    min-height: 66px;
    padding: 5px 12px !important;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }

  .sidebar-title {
    color: var(--text-secondary);
  }

  .sidebar-content {
    scrollbar-width: thin;
    scrollbar-color: var(--scrollbar-thumb) var(--scrollbar-track);
  }

  .sidebar-content::-webkit-scrollbar {
    width: 8px;
  }

  .sidebar-content::-webkit-scrollbar-track {
    background: var(--scrollbar-track);
  }

  .sidebar-content::-webkit-scrollbar-thumb {
    background: var(--scrollbar-thumb);
    border-radius: 4px;
  }

  .sidebar-content::-webkit-scrollbar-thumb:hover {
    background: var(--scrollbar-thumb-hover);
  }

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
    color: var(--text-muted);
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
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .tree-label {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 4px;
    background: transparent !important;
    border: none;
    color: var(--text-primary) !important;
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
    background: var(--hover-bg) !important;
  }

  .tree-label.active {
    background: var(--selected-bg) !important;
    color: var(--accent-blue) !important;
    font-weight: 500;
  }

  .tree-icon {
    font-size: 11px;
    width: 16px;
    text-align: center;
    flex-shrink: 0;
    color: var(--text-muted) !important;
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
    color: var(--accent-red) !important;
  }

  .connection-status-badge {
    position: absolute;
    bottom: -2px;
    left: -2px;
    font-size: 8px;
    color: var(--accent-green);
    background: var(--bg-secondary);
    border-radius: 50%;
  }

  .tree-label.active .tree-icon {
    color: var(--accent-blue) !important;
  }

  .tree-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .connection-details {
    font-size: 12px;
    color: var(--text-muted);
    font-weight: normal;
    margin-left: 8px;
    flex-shrink: 0;
  }

  .tree-label.active .connection-details {
    color: var(--accent-blue-hover);
  }

  .tree-badge {
    font-size: 10px;
    padding: 2px 6px;
    background: var(--bg-tertiary);
    border-radius: 3px;
    color: var(--text-muted);
    font-weight: 500;
    flex-shrink: 0;
    margin-left: auto;
  }

  .tree-label.active .tree-badge {
    background: var(--accent-blue-light);
  }

  .tree-children {
    margin-left: 12px;
  }

  .tree-section-header {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 4px;
    background: transparent !important;
    border: none;
    color: var(--text-primary) !important;
    padding: 2px 8px;
    cursor: pointer;
    font-size: 12px;
    text-align: left;
    transition: background-color 0.15s;
    min-height: 20px;
    border-radius: 3px;
    line-height: 1.5;
  }

  .tree-section-header:hover {
    background: var(--hover-bg) !important;
  }

  .tree-section-header.active {
    background: var(--selected-bg) !important;
    color: var(--accent-blue) !important;
    font-weight: 500;
  }

  .tree-section-header i {
    font-size: 11px;
    width: 16px;
    text-align: center;
    flex-shrink: 0;
    color: var(--text-muted) !important;
  }

  .tree-section-header.active i {
    color: var(--accent-blue) !important;
  }

  .database-node {
    padding-left: 8px;
  }

  .tables-section-node {
    padding-left: 8px;
  }

  /* Loading spinner animation */
  .fa-spinner {
    color: var(--accent-blue);
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
    background-color: var(--accent-blue-light) !important;
  }

  /* Table/Collection/Cache item row styling - consistent with tree structure */
  .table-item-row {
    background: transparent !important;
  }

  .table-item-row:hover {
    background: var(--hover-bg) !important;
  }

  .table-item-row.table-active {
    background: var(--selected-bg) !important;
  }

  .table-item-row td {
    background: transparent !important;
  }

  /* Table/Collection/Cache item button styling - consistent with tree-label */
  .table-item-row .btn.text-start {
    color: var(--text-primary) !important;
    background: transparent !important;
  }

  .table-item-row .btn.text-start:hover {
    background: transparent !important;
  }

  .table-item-row.table-active .btn.text-start {
    color: var(--accent-blue) !important;
    font-weight: 500;
  }

  /* Icon color for table items - consistent with tree-icon */
  .table-item-row .fa-table,
  .table-item-row .fa-server,
  .table-item-row .fa-layer-group {
    color: var(--text-muted) !important;
  }

  .table-item-row.table-active .fa-table,
  .table-item-row.table-active .fa-server,
  .table-item-row.table-active .fa-layer-group {
    color: var(--accent-blue) !important;
  }

  /* Chevron button styling */
  .table-item-row .btn.text-secondary {
    color: var(--text-muted) !important;
    background: transparent !important;
  }

  /* MySQL Object Items Styling (Views, Indexes, Procedures, Triggers, Events) */
  .mysql-object-table {
    background: transparent !important;
  }

  .mysql-object-table .table-item-row {
    background: transparent !important;
  }

  .mysql-object-table .table-item-row:hover {
    background: var(--hover-bg) !important;
  }

  .mysql-object-btn {
    color: var(--text-primary) !important;
    background: transparent !important;
  }

  .mysql-object-btn:hover {
    background: transparent !important;
  }

  .mysql-object-icon {
    color: var(--text-muted) !important;
  }

  .mysql-object-table .table-item-row:hover .mysql-object-icon {
    color: var(--text-primary) !important;
  }
</style>
