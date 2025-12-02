<script>
  import { onMount, createEventDispatcher } from "svelte";
  import TreeView from "./tree/TreeView.svelte";
  import ConnectionItem from "./tree/ConnectionItem.svelte";
  import DatabaseItem from "./tree/DatabaseItem.svelte";
  import SchemaItem from "./tree/SchemaItem.svelte";
  import ObjectGroup from "./tree/ObjectGroup.svelte";
  import ObjectItem from "./tree/ObjectItem.svelte";

  // Stores
  import {
    connections,
    activeConnection,
    selectedDatabase,
    selectedTable,
  } from "../../../stores/connections";

  // Utils
  import {
    getConnectionsInfo,
    getDatabaseObject,
    getConnectedDatabases,
    connectToDatabase,
    disconnectFromDatabase,
    deleteConnection,
    getConnectionForEdit,
    saveConnection,
  } from "../../../utils/tauri";

  // Context Menus
  import ConnectionContextMenu from "../../context-menus/ConnectionContextMenu.svelte";
  import DatabaseContextMenu from "../../context-menus/DatabaseContextMenu.svelte";
  import SchemaContextMenu from "../../context-menus/SchemaContextMenu.svelte";
  import TableContextMenu from "../../context-menus/TableContextMenu.svelte";
  import ViewContextMenu from "../../context-menus/ViewContextMenu.svelte";
  import InputModal from "../../modals/InputModal.svelte";
  import ConnectionModal from "../../modals/ConnectionModal.svelte";

  const dispatch = createEventDispatcher();

  // State
  let searchQuery = "";
  let expandedConnections = {};
  let expandedDatabases = {};
  let expandedSchemas = {};
  let expandedSchemasParent = {};
  let expandedGroups = {}; // For object groups (tables, views, etc.)

  // Loading states
  let loadingConnections = {};
  let loadingDatabases = {};
  let loadingSchemas = {};

  // Connection status
  let connectedConnections = {};

  // Cache for data
  let cachedData = {}; // { 'key': { tables, views, indexes, procedures, triggers, events, schemas } }

  // Context menus
  let contextMenu = null;
  let databaseContextMenu = null;
  let schemaContextMenu = null;
  let tableContextMenu = null;
  let viewContextMenu = null;

  // Active context items
  let activeContextItem = null;

  // Modals
  let showConnectionModal = false;
  let editingConnection = null;
  let showRenameModal = false;
  let renameModalTitle = "";
  let renameModalValue = "";
  let renameModalCallback = null;

  // Lifecycle
  onMount(async () => {
    await loadConnections();
    await syncConnectedStatus();

    // Close context menus on click
    const closeMenus = () => {
      contextMenu = null;
      databaseContextMenu = null;
      schemaContextMenu = null;
      tableContextMenu = null;
      viewContextMenu = null;
      activeContextItem = null;
    };

    document.addEventListener("click", closeMenus);
    return () => document.removeEventListener("click", closeMenus);
  });

  // Data loading
  async function loadConnections() {
    try {
      const conns = await getConnectionsInfo();
      connections.set(conns);
    } catch (error) {
      console.error("Failed to load connections:", error);
    }
  }

  async function syncConnectedStatus() {
    try {
      const connectedIds = await getConnectedDatabases();
      connectedConnections = connectedIds.reduce((acc, id) => {
        acc[id] = true;
        return acc;
      }, {});
    } catch (error) {
      console.error("Failed to sync connected status:", error);
    }
  }

  // Filter connections
  $: filteredConnections = ($connections || []).filter((conn) => {
    if (!conn?.name) return false;
    const query = (searchQuery || "").toLowerCase();
    return conn.name.toLowerCase().includes(query);
  });

  // Toggle handlers
  async function handleConnectionToggle(e) {
    const conn = e.detail;
    const isExpanded = expandedConnections[conn.id];

    if (!isExpanded) {
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

  async function handleDatabaseToggle(e, conn) {
    const db = e.detail;
    const key = `${conn.id}-${db.name}`;
    const isExpanded = expandedDatabases[key];

    if (!isExpanded) {
      const connObj = $connections.find((c) => c.id === conn.id);
      if (connObj) activeConnection.set(connObj);
      selectedDatabase.set(db);

      loadingDatabases[key] = true;
      loadingDatabases = { ...loadingDatabases };

      try {
        if (conn.db_type === "PostgreSQL") {
          // PostgreSQL: Load schemas
          const result = await getDatabaseObject(
            conn.id,
            "schema_list",
            db.name
          );
          cachedData[key] = { schemas: result.schemas || [] };
        } else {
          // MySQL/MSSQL/Others: Load database info
          const result = await getDatabaseObject(
            conn.id,
            "database_info",
            db.name
          );
          cachedData[key] = {
            tables: result.tables || [],
            views: result.views || [],
            indexes: result.indexes || [],
            procedures: result.procedures || [],
            triggers: result.triggers || [],
            events: result.events || [],
          };
        }
        expandedDatabases[key] = true;
      } catch (error) {
        console.error("Failed to load database:", error);
      } finally {
        loadingDatabases[key] = false;
        loadingDatabases = { ...loadingDatabases };
      }
    } else {
      delete expandedDatabases[key];
    }
    expandedDatabases = { ...expandedDatabases };
  }

  async function handleSchemaToggle(e, conn, db) {
    const { schemaName } = e.detail;
    const key = `${conn.id}-${db.name}-${schemaName}`;
    const isExpanded = expandedSchemas[key];

    if (!isExpanded) {
      loadingSchemas[key] = true;
      loadingSchemas = { ...loadingSchemas };

      try {
        const result = await getDatabaseObject(
          conn.id,
          "schema_info",
          db.name,
          schemaName
        );
        cachedData[key] = {
          tables: result.tables || [],
          views: result.views || [],
          indexes: result.indexes || [],
          procedures: result.procedures || [],
          triggers: result.triggers || [],
        };
        expandedSchemas[key] = true;
      } catch (error) {
        console.error("Failed to load schema:", error);
      } finally {
        loadingSchemas[key] = false;
        loadingSchemas = { ...loadingSchemas };
      }
    } else {
      delete expandedSchemas[key];
    }
    expandedSchemas = { ...expandedSchemas };
  }

  function handleGroupToggle(groupKey) {
    expandedGroups[groupKey] = !expandedGroups[groupKey];
    expandedGroups = { ...expandedGroups };
  }

  function handleSchemasParentToggle(key) {
    expandedSchemasParent[key] = !expandedSchemasParent[key];
    expandedSchemasParent = { ...expandedSchemasParent };
  }

  // Item handlers
  function handleTableClick(table, conn, db, schema = null) {
    selectedTable.set({
      ...table,
      _connId: conn.id,
      _dbName: db.name,
      _schema: schema || table.schema || null,
    });

    const connObj = $connections.find((c) => c.id === conn.id);
    if (connObj) activeConnection.set(connObj);
    selectedDatabase.set(db);
  }

  function handleTableDblClick(table, conn, db, schema = null) {
    // Ensure schema is included in the table object
    const tableWithSchema = {
      ...table,
      schema: schema || table.schema || null,
    };
    dispatch("openTableTab", {
      table: tableWithSchema,
      database: db,
      connection: conn,
    });
  }

  function handleViewDblClick(view, conn, db, schema = null) {
    dispatch("openTableTab", {
      table: { name: view.name, schema: schema || view.schema, isView: true },
      database: db,
      connection: conn,
    });
  }

  function handleProcedureDblClick(proc, conn, db, schema = null) {
    dispatch("openProcedureTab", {
      procedure: { ...proc, schema: schema || proc.schema },
      database: db,
      connection: conn,
    });
  }

  // Context menu handlers
  function handleConnectionContextMenu(e) {
    const { event, connection } = e.detail;
    event.preventDefault();
    event.stopPropagation();
    activeContextItem = `conn-${connection.id}`;
    contextMenu = { x: event.clientX, y: event.clientY, connection };
  }

  function handleDatabaseContextMenu(e, conn) {
    const { event, database } = e.detail;
    event.preventDefault();
    event.stopPropagation();
    activeContextItem = `db-${conn.id}-${database.name}`;
    databaseContextMenu = {
      x: event.clientX,
      y: event.clientY,
      database,
      connection: conn,
    };
  }

  function handleSchemaContextMenu(e, conn, db) {
    const { event, schemaName } = e.detail;
    event.preventDefault();
    event.stopPropagation();
    activeContextItem = `schema-${conn.id}-${db.name}-${schemaName}`;
    schemaContextMenu = {
      x: event.clientX,
      y: event.clientY,
      schema: schemaName,
      database: db,
      connection: conn,
    };
  }

  function handleTableContextMenu(e, conn, db) {
    const { event, item } = e.detail;
    event.preventDefault();
    event.stopPropagation();
    activeContextItem = `table-${conn.id}-${db.name}-${item.name}`;
    tableContextMenu = {
      x: event.clientX,
      y: event.clientY,
      table: item,
      database: db,
      connection: conn,
    };
  }

  function handleViewContextMenu(e, conn, db) {
    const { event, item } = e.detail;
    event.preventDefault();
    event.stopPropagation();
    activeContextItem = `view-${conn.id}-${db.name}-${item.name}`;
    viewContextMenu = {
      x: event.clientX,
      y: event.clientY,
      view: item,
      database: db,
      connection: conn,
    };
  }

  // Connection context menu handlers
  async function handleConnectionEdit(e) {
    const conn = e.detail;
    try {
      const fullConfig = await getConnectionForEdit(conn.id);
      editingConnection = fullConfig;
      showConnectionModal = true;
    } catch (error) {
      console.error("Failed to load connection for edit:", error);
    }
    contextMenu = null;
  }

  async function handleConnectionDelete(e) {
    const conn = e.detail;
    if (confirm(`Are you sure you want to delete connection "${conn.name}"?`)) {
      try {
        await deleteConnection(conn.id);
        await loadConnections();
      } catch (error) {
        console.error("Failed to delete connection:", error);
        alert(`Failed to delete connection: ${error}`);
      }
    }
    contextMenu = null;
  }

  async function handleConnectionRefresh(e) {
    const conn = e.detail;
    if (connectedConnections[conn.id]) {
      // Reconnect
      try {
        await disconnectFromDatabase(conn.id);
        await connectToDatabase(conn.id);
        await syncConnectedStatus();

        // Reload data if connection was expanded
        if (expandedConnections[conn.id]) {
          expandedConnections[conn.id] = null;
          await handleConnectionToggle({ detail: conn });
        }
      } catch (error) {
        console.error("Failed to reconnect:", error);
        alert(`Failed to reconnect: ${error}`);
      }
    }
    contextMenu = null;
  }

  async function handleConnectionConnect(e) {
    const conn = e.detail;
    try {
      await connectToDatabase(conn.id);
      await syncConnectedStatus();

      // Auto-expand after connecting
      if (!expandedConnections[conn.id]) {
        await handleConnectionToggle({ detail: conn });
      }
    } catch (error) {
      console.error("Failed to connect:", error);
      alert(`Failed to connect: ${error}`);
    }
    contextMenu = null;
  }

  async function handleConnectionDisconnect(e) {
    const conn = e.detail;
    try {
      await disconnectFromDatabase(conn.id);
      await syncConnectedStatus();

      // Collapse after disconnecting
      if (expandedConnections[conn.id]) {
        expandedConnections[conn.id] = null;
        expandedConnections = { ...expandedConnections };
      }
    } catch (error) {
      console.error("Failed to disconnect:", error);
      alert(`Failed to disconnect: ${error}`);
    }
    contextMenu = null;
  }

  function handleConnectionCopy(e) {
    const conn = e.detail;
    const connectionInfo = `Name: ${conn.name}\nType: ${conn.db_type}\nHost: ${conn.host}\nPort: ${conn.port}`;
    navigator.clipboard.writeText(connectionInfo).then(
      () => console.log("Connection info copied to clipboard"),
      (err) => console.error("Failed to copy:", err)
    );
    contextMenu = null;
  }

  function handleConnectionRename(e) {
    const conn = e.detail;
    renameModalTitle = "Rename Connection";
    renameModalValue = conn.name;
    renameModalCallback = async (newName) => {
      try {
        const fullConfig = await getConnectionForEdit(conn.id);
        fullConfig.name = newName;
        await saveConnection(fullConfig);
        await loadConnections();
      } catch (error) {
        console.error("Failed to rename connection:", error);
        alert(`Failed to rename connection: ${error}`);
      }
    };
    showRenameModal = true;
    contextMenu = null;
  }

  // Database context menu handlers
  function handleDatabaseSqlEditor(e) {
    const { database, connection } = e.detail;
    // Open SQL Editor tab for this database
    dispatch("openSqlEditorTab", { database, connection });
    databaseContextMenu = null;
  }

  function handleDatabaseView(e) {
    const { database, connection } = e.detail;
    // Open database view/properties
    console.log("View database:", database.name);
    // TODO: Implement database view modal
    databaseContextMenu = null;
  }

  function handleDatabaseCopy(e) {
    const { database, connection } = e.detail;
    const databaseInfo = `Database: ${database.name}\nConnection: ${connection.name}\nType: ${connection.db_type}`;
    navigator.clipboard.writeText(databaseInfo).then(
      () => console.log("Database info copied to clipboard"),
      (err) => console.error("Failed to copy:", err)
    );
    databaseContextMenu = null;
  }

  function handleDatabasePaste(e) {
    const { database, connection } = e.detail;
    // Paste functionality - to be implemented
    console.log("Paste to database:", database.name);
    // TODO: Implement paste functionality
    databaseContextMenu = null;
  }

  function handleDatabaseCopyAdvancedInfo(e) {
    const { database, connection } = e.detail;
    const advancedInfo = `Database Name: ${database.name}\nConnection: ${connection.name}\nConnection ID: ${connection.id}\nDB Type: ${connection.db_type}\nHost: ${connection.host}\nPort: ${connection.port}`;
    navigator.clipboard.writeText(advancedInfo).then(
      () => console.log("Advanced database info copied to clipboard"),
      (err) => console.error("Failed to copy:", err)
    );
    databaseContextMenu = null;
  }

  function handleDatabaseDelete(e) {
    const { database, connection } = e.detail;
    if (
      confirm(
        `Are you sure you want to delete database "${database.name}"?\n\nWARNING: This will permanently delete all data in this database!`
      )
    ) {
      console.log("Delete database:", database.name);
      // TODO: Implement database deletion (requires backend command)
      alert("Database deletion is not yet implemented.");
    }
    databaseContextMenu = null;
  }

  function handleDatabaseRename(e) {
    const { database, connection } = e.detail;
    renameModalTitle = "Rename Database";
    renameModalValue = database.name;
    renameModalCallback = async (newName) => {
      console.log("Rename database from", database.name, "to", newName);
      // TODO: Implement database rename (requires backend command)
      alert("Database rename is not yet implemented.");
    };
    showRenameModal = true;
    databaseContextMenu = null;
  }

  async function handleDatabaseRefresh(e) {
    const { database, connection } = e.detail;
    // Refresh database data
    const dbKey = `${connection.id}-${database.name}`;
    try {
      // Clear cache
      if (cachedData[dbKey]) {
        cachedData[dbKey] = {};
      }

      // Reload if expanded
      if (expandedDatabases[dbKey]) {
        expandedDatabases[dbKey] = null;
        await handleDatabaseToggle({ detail: database }, connection);
      }
    } catch (error) {
      console.error("Failed to refresh database:", error);
    }
    databaseContextMenu = null;
  }

  // Schema context menu handlers
  function handleSchemaSqlEditor(e) {
    const { schema, database, connection } = e.detail;
    dispatch("openSqlEditorTab", { schema, database, connection });
    schemaContextMenu = null;
  }

  function handleSchemaView(e) {
    const { schema, database, connection } = e.detail;
    console.log("View schema:", schema);
    // TODO: Implement schema view modal
    schemaContextMenu = null;
  }

  function handleSchemaViewDiagram(e) {
    const { schema, database, connection } = e.detail;
    console.log("View diagram for schema:", schema);
    // TODO: Implement schema diagram view
    schemaContextMenu = null;
  }

  function handleSchemaImportData(e) {
    const { schema, database, connection } = e.detail;
    console.log("Import data to schema:", schema);
    // TODO: Implement import data functionality
    schemaContextMenu = null;
  }

  function handleSchemaGenerateSql(e) {
    const { schema, database, connection } = e.detail;
    console.log("Generate SQL for schema:", schema);
    // TODO: Implement SQL generation
    schemaContextMenu = null;
  }

  function handleSchemaCopy(e) {
    const { schema, database, connection } = e.detail;
    const schemaInfo = `Schema: ${schema}\nDatabase: ${database.name}\nConnection: ${connection.name}`;
    navigator.clipboard.writeText(schemaInfo).then(
      () => console.log("Schema info copied to clipboard"),
      (err) => console.error("Failed to copy:", err)
    );
    schemaContextMenu = null;
  }

  function handleSchemaPaste(e) {
    const { schema, database, connection } = e.detail;
    console.log("Paste to schema:", schema);
    // TODO: Implement paste functionality
    schemaContextMenu = null;
  }

  function handleSchemaCopyAdvancedInfo(e) {
    const { schema, database, connection } = e.detail;
    const advancedInfo = `Schema: ${schema}\nDatabase: ${database.name}\nConnection: ${connection.name}\nConnection ID: ${connection.id}\nDB Type: ${connection.db_type}`;
    navigator.clipboard.writeText(advancedInfo).then(
      () => console.log("Advanced schema info copied to clipboard"),
      (err) => console.error("Failed to copy:", err)
    );
    schemaContextMenu = null;
  }

  function handleSchemaDelete(e) {
    const { schema, database, connection } = e.detail;
    if (confirm(`Are you sure you want to delete schema "${schema}"?`)) {
      console.log("Delete schema:", schema);
      // TODO: Implement schema deletion
      alert("Schema deletion is not yet implemented.");
    }
    schemaContextMenu = null;
  }

  function handleSchemaRename(e) {
    const { schema, database, connection } = e.detail;
    renameModalTitle = "Rename Schema";
    renameModalValue = schema;
    renameModalCallback = async (newName) => {
      console.log("Rename schema from", schema, "to", newName);
      // TODO: Implement schema rename
      alert("Schema rename is not yet implemented.");
    };
    showRenameModal = true;
    schemaContextMenu = null;
  }

  async function handleSchemaRefresh(e) {
    const { schema, database, connection } = e.detail;
    const schemaKey = `${connection.id}-${database.name}-${schema}`;
    try {
      // Clear cache for this schema
      if (cachedData[schemaKey]) {
        cachedData[schemaKey] = {};
      }

      // Reload if expanded
      if (expandedSchemas[schemaKey]) {
        expandedSchemas[schemaKey] = null;
        await handleSchemaToggle(
          { detail: { schemaName: schema } },
          connection,
          database
        );
      }
    } catch (error) {
      console.error("Failed to refresh schema:", error);
    }
    schemaContextMenu = null;
  }

  // Table context menu handlers
  function handleTableViewTable(e) {
    const { table, database, connection } = e.detail;
    console.log("View table structure:", table.name);
    // TODO: Implement table structure view
    tableContextMenu = null;
  }

  function handleTableViewDiagram(e) {
    const { table, database, connection } = e.detail;
    console.log("View diagram for table:", table.name);
    // TODO: Implement table diagram
    tableContextMenu = null;
  }

  function handleTableViewData(e) {
    const { table, database, connection } = e.detail;
    handleTableDblClick(table, connection, database, table.schema);
    tableContextMenu = null;
  }

  function handleTableExportData(e) {
    const { table, database, connection } = e.detail;
    console.log("Export data from table:", table.name);
    // TODO: Implement data export
    tableContextMenu = null;
  }

  function handleTableImportData(e) {
    const { table, database, connection } = e.detail;
    console.log("Import data to table:", table.name);
    // TODO: Implement data import
    tableContextMenu = null;
  }

  function handleTableReadInConsole(e) {
    const { table, database, connection } = e.detail;
    const schema = table.schema ? `${table.schema}.` : "";
    const query = `SELECT * FROM ${schema}${table.name} LIMIT 100;`;
    dispatch("openSqlEditorTab", { database, connection, initialQuery: query });
    tableContextMenu = null;
  }

  function handleTableCopy(e) {
    const { table, database, connection } = e.detail;
    const tableInfo = `Table: ${table.name}\nDatabase: ${database.name}\nConnection: ${connection.name}`;
    navigator.clipboard.writeText(tableInfo).then(
      () => console.log("Table info copied to clipboard"),
      (err) => console.error("Failed to copy:", err)
    );
    tableContextMenu = null;
  }

  function handleTablePaste(e) {
    const { table, database, connection } = e.detail;
    console.log("Paste to table:", table.name);
    // TODO: Implement paste functionality
    tableContextMenu = null;
  }

  function handleTableCopyAdvancedInfo(e) {
    const { table, database, connection } = e.detail;
    const advancedInfo = `Table: ${table.name}\nSchema: ${table.schema || "N/A"}\nDatabase: ${database.name}\nConnection: ${connection.name}\nConnection ID: ${connection.id}\nDB Type: ${connection.db_type}`;
    navigator.clipboard.writeText(advancedInfo).then(
      () => console.log("Advanced table info copied to clipboard"),
      (err) => console.error("Failed to copy:", err)
    );
    tableContextMenu = null;
  }

  function handleTableDelete(e) {
    const { table, database, connection } = e.detail;
    if (
      confirm(
        `Are you sure you want to delete table "${table.name}"?\n\nWARNING: This will permanently delete all data in this table!`
      )
    ) {
      console.log("Delete table:", table.name);
      // TODO: Implement table deletion
      alert("Table deletion is not yet implemented.");
    }
    tableContextMenu = null;
  }

  function handleTableRename(e) {
    const { table, database, connection } = e.detail;
    renameModalTitle = "Rename Table";
    renameModalValue = table.name;
    renameModalCallback = async (newName) => {
      console.log("Rename table from", table.name, "to", newName);
      // TODO: Implement table rename
      alert("Table rename is not yet implemented.");
    };
    showRenameModal = true;
    tableContextMenu = null;
  }

  async function handleTableRefresh(e) {
    const { table, database, connection } = e.detail;
    const dbKey = `${connection.id}-${database.name}`;
    try {
      // Reload tables for this database
      const response = await getDatabaseObject(
        connection.id,
        "database_info",
        database.name
      );
      if (response && response.tables) {
        cachedData[dbKey] = {
          ...cachedData[dbKey],
          tables: response.tables,
        };
        cachedData = { ...cachedData };
      }
    } catch (error) {
      console.error("Failed to refresh table:", error);
    }
    tableContextMenu = null;
  }

  // View context menu handlers
  function handleViewStructure(e) {
    const { view, database, connection } = e.detail;
    console.log("View structure:", view.name);
    // TODO: Implement view structure modal
    viewContextMenu = null;
  }

  function handleViewDefinition(e) {
    const { view, database, connection } = e.detail;
    console.log("View definition:", view.name);
    // TODO: Implement view definition modal
    viewContextMenu = null;
  }

  function handleViewData(e) {
    const { view, database, connection } = e.detail;
    handleViewDblClick(view, connection, database, view.schema);
    viewContextMenu = null;
  }

  function handleViewExportData(e) {
    const { view, database, connection } = e.detail;
    console.log("Export data from view:", view.name);
    // TODO: Implement data export
    viewContextMenu = null;
  }

  function handleViewImportData(e) {
    const { view, database, connection } = e.detail;
    console.log("Import data to view:", view.name);
    // TODO: Implement data import
    viewContextMenu = null;
  }

  function handleViewReadInConsole(e) {
    const { view, database, connection } = e.detail;
    const schema = view.schema ? `${view.schema}.` : "";
    const query = `SELECT * FROM ${schema}${view.name} LIMIT 100;`;
    dispatch("openSqlEditorTab", { database, connection, initialQuery: query });
    viewContextMenu = null;
  }

  function handleViewCopy(e) {
    const { view, database, connection } = e.detail;
    const viewInfo = `View: ${view.name}\nDatabase: ${database.name}\nConnection: ${connection.name}`;
    navigator.clipboard.writeText(viewInfo).then(
      () => console.log("View info copied to clipboard"),
      (err) => console.error("Failed to copy:", err)
    );
    viewContextMenu = null;
  }

  function handleViewCopyAdvancedInfo(e) {
    const { view, database, connection } = e.detail;
    const advancedInfo = `View: ${view.name}\nSchema: ${view.schema || "N/A"}\nDatabase: ${database.name}\nConnection: ${connection.name}\nConnection ID: ${connection.id}\nDB Type: ${connection.db_type}`;
    navigator.clipboard.writeText(advancedInfo).then(
      () => console.log("Advanced view info copied to clipboard"),
      (err) => console.error("Failed to copy:", err)
    );
    viewContextMenu = null;
  }

  function handleViewRename(e) {
    const { view, database, connection } = e.detail;
    renameModalTitle = "Rename View";
    renameModalValue = view.name;
    renameModalCallback = async (newName) => {
      console.log("Rename view from", view.name, "to", newName);
      // TODO: Implement view rename
      alert("View rename is not yet implemented.");
    };
    showRenameModal = true;
    viewContextMenu = null;
  }

  function handleViewDelete(e) {
    const { view, database, connection } = e.detail;
    if (confirm(`Are you sure you want to delete view "${view.name}"?`)) {
      console.log("Delete view:", view.name);
      // TODO: Implement view deletion
      alert("View deletion is not yet implemented.");
    }
    viewContextMenu = null;
  }

  async function handleViewRefresh(e) {
    const { view, database, connection } = e.detail;
    const dbKey = `${connection.id}-${database.name}`;
    try {
      // Reload views for this database
      const response = await getDatabaseObject(
        connection.id,
        "database_info",
        database.name
      );
      if (response && response.views) {
        cachedData[dbKey] = {
          ...cachedData[dbKey],
          views: response.views,
        };
        cachedData = { ...cachedData };
      }
    } catch (error) {
      console.error("Failed to refresh view:", error);
    }
    viewContextMenu = null;
  }

  // Connection actions
  function openNewConnectionModal() {
    editingConnection = null;
    showConnectionModal = true;
  }

  async function handleSaveConnection() {
    showConnectionModal = false;
    editingConnection = null;
    await loadConnections();
  }

  // Helper to get group key
  function getGroupKey(connId, dbName, type, schemaName = null) {
    return schemaName
      ? `${connId}-${dbName}-${schemaName}-${type}`
      : `${connId}-${dbName}-${type}`;
  }

  // Helper to get data for a key
  function getData(key, type) {
    return cachedData[key]?.[type] || [];
  }

  // Get counts
  function getCount(key, type) {
    return cachedData[key]?.[type]?.length || 0;
  }

  // MSSQL schema grouping helper
  function groupBySchema(tables) {
    return tables.reduce((acc, table) => {
      const schema = table.schema || "dbo";
      if (!acc[schema]) acc[schema] = [];
      acc[schema].push(table);
      return acc;
    }, {});
  }
</script>

<div class="sidebar2-container">
  <TreeView
    searchable={true}
    searchPlaceholder="Search connection or database"
    title="Connections"
    bind:searchQuery
  >
    <i slot="title-icon" class="fas fa-network-wired"></i>

    <button
      slot="header-actions"
      class="btn btn-sm btn-success"
      on:click={openNewConnectionModal}
      title="Add Connection"
    >
      <i class="fas fa-plus"></i>
    </button>

    <!-- Connections List -->
    {#if filteredConnections.length === 0}
      <p class="text-muted small p-3">No connections found</p>
    {/if}

    {#each filteredConnections as conn (conn.id)}
      <ConnectionItem
        connection={conn}
        expanded={!!expandedConnections[conn.id]}
        loading={loadingConnections[conn.id]}
        connected={connectedConnections[conn.id]}
        active={activeContextItem === `conn-${conn.id}`}
        on:toggle={handleConnectionToggle}
        on:contextmenu={handleConnectionContextMenu}
      >
        <!-- Databases for this connection -->
        {#if expandedConnections[conn.id]}
          {#if conn.db_type === "Ignite"}
            <!-- Ignite: Direct caches -->
            {#each expandedConnections[conn.id].databases || [] as cache (cache.name)}
              <ObjectItem
                item={cache}
                name={cache.name}
                type="cache"
                indent={1}
                active={$selectedTable?.name === cache.name &&
                  $selectedTable?._connId === conn.id}
                on:click={() =>
                  handleTableClick({ name: cache.name, schema: null }, conn, {
                    name: cache.name,
                  })}
                on:dblclick={() =>
                  handleTableDblClick(
                    { name: cache.name, schema: null },
                    conn,
                    { name: cache.name }
                  )}
              />
            {/each}
          {:else}
            <!-- Other DBs: Database list -->
            {#each expandedConnections[conn.id].databases || [] as db (db.name)}
              {@const dbKey = `${conn.id}-${db.name}`}
              <DatabaseItem
                database={db}
                expanded={!!expandedDatabases[dbKey]}
                loading={loadingDatabases[dbKey]}
                active={activeContextItem === `db-${dbKey}`}
                on:toggle={(e) => handleDatabaseToggle(e, conn)}
                on:contextmenu={(e) => handleDatabaseContextMenu(e, conn)}
              >
                {#if expandedDatabases[dbKey]}
                  {#if conn.db_type === "PostgreSQL"}
                    <!-- PostgreSQL: Schemas parent -->
                    <ObjectGroup
                      type="schemas"
                      count={cachedData[dbKey]?.schemas?.length || 0}
                      expanded={!!expandedSchemasParent[dbKey]}
                      indent={2}
                      on:toggle={() => handleSchemasParentToggle(dbKey)}
                    >
                      {#if expandedSchemasParent[dbKey]}
                        {#each cachedData[dbKey]?.schemas || [] as schema (schema.name)}
                          {@const schemaKey = `${dbKey}-${schema.name}`}
                          <SchemaItem
                            {schema}
                            schemaName={schema.name}
                            expanded={!!expandedSchemas[schemaKey]}
                            loading={loadingSchemas[schemaKey]}
                            active={activeContextItem === `schema-${schemaKey}`}
                            indent={3}
                            icon="folder"
                            on:toggle={(e) => handleSchemaToggle(e, conn, db)}
                            on:contextmenu={(e) =>
                              handleSchemaContextMenu(e, conn, db)}
                          >
                            {#if expandedSchemas[schemaKey]}
                              <!-- Tables -->
                              <ObjectGroup
                                type="tables"
                                count={getCount(schemaKey, "tables")}
                                expanded={!!expandedGroups[
                                  getGroupKey(
                                    conn.id,
                                    db.name,
                                    "tables",
                                    schema.name
                                  )
                                ]}
                                indent={4}
                                on:toggle={() =>
                                  handleGroupToggle(
                                    getGroupKey(
                                      conn.id,
                                      db.name,
                                      "tables",
                                      schema.name
                                    )
                                  )}
                              >
                                {#if expandedGroups[getGroupKey(conn.id, db.name, "tables", schema.name)]}
                                  {#each getData(schemaKey, "tables") as table (table.name)}
                                    <ObjectItem
                                      item={table}
                                      name={table.name}
                                      type="table"
                                      size={table.size}
                                      indent={5}
                                      active={$selectedTable?.name ===
                                        table.name &&
                                        $selectedTable?._connId === conn.id &&
                                        $selectedTable?._schema === schema.name}
                                      on:click={() =>
                                        handleTableClick(
                                          table,
                                          conn,
                                          db,
                                          schema.name
                                        )}
                                      on:dblclick={() =>
                                        handleTableDblClick(
                                          table,
                                          conn,
                                          db,
                                          schema.name
                                        )}
                                      on:contextmenu={(e) =>
                                        handleTableContextMenu(e, conn, db)}
                                    />
                                  {/each}
                                {/if}
                              </ObjectGroup>

                              <!-- Views -->
                              <ObjectGroup
                                type="views"
                                count={getCount(schemaKey, "views")}
                                expanded={!!expandedGroups[
                                  getGroupKey(
                                    conn.id,
                                    db.name,
                                    "views",
                                    schema.name
                                  )
                                ]}
                                indent={4}
                                on:toggle={() =>
                                  handleGroupToggle(
                                    getGroupKey(
                                      conn.id,
                                      db.name,
                                      "views",
                                      schema.name
                                    )
                                  )}
                              >
                                {#if expandedGroups[getGroupKey(conn.id, db.name, "views", schema.name)]}
                                  {#each getData(schemaKey, "views") as view (view.name)}
                                    <ObjectItem
                                      item={{ ...view, schema: schema.name }}
                                      name={view.name}
                                      type="view"
                                      indent={5}
                                      on:dblclick={() =>
                                        handleViewDblClick(
                                          { ...view, schema: schema.name },
                                          conn,
                                          db,
                                          schema.name
                                        )}
                                      on:contextmenu={(e) =>
                                        handleViewContextMenu(e, conn, db)}
                                    />
                                  {/each}
                                {/if}
                              </ObjectGroup>

                              <!-- Indexes -->
                              <ObjectGroup
                                type="indexes"
                                count={getCount(schemaKey, "indexes")}
                                expanded={!!expandedGroups[
                                  getGroupKey(
                                    conn.id,
                                    db.name,
                                    "indexes",
                                    schema.name
                                  )
                                ]}
                                indent={4}
                                on:toggle={() =>
                                  handleGroupToggle(
                                    getGroupKey(
                                      conn.id,
                                      db.name,
                                      "indexes",
                                      schema.name
                                    )
                                  )}
                              >
                                {#if expandedGroups[getGroupKey(conn.id, db.name, "indexes", schema.name)]}
                                  {#each getData(schemaKey, "indexes") as idx (`${idx.table_name}-${idx.name}`)}
                                    <ObjectItem
                                      item={idx}
                                      name="{idx.table_name}.{idx.name}"
                                      type="index"
                                      badge={idx.is_unique ? "U" : "I"}
                                      badgeType={idx.is_unique
                                        ? "info"
                                        : "secondary"}
                                      indent={5}
                                    />
                                  {/each}
                                {/if}
                              </ObjectGroup>

                              <!-- Functions -->
                              <ObjectGroup
                                type="functions"
                                count={getCount(schemaKey, "procedures")}
                                expanded={!!expandedGroups[
                                  getGroupKey(
                                    conn.id,
                                    db.name,
                                    "functions",
                                    schema.name
                                  )
                                ]}
                                indent={4}
                                on:toggle={() =>
                                  handleGroupToggle(
                                    getGroupKey(
                                      conn.id,
                                      db.name,
                                      "functions",
                                      schema.name
                                    )
                                  )}
                              >
                                {#if expandedGroups[getGroupKey(conn.id, db.name, "functions", schema.name)]}
                                  {#each getData(schemaKey, "procedures") as proc (proc.oid || proc.name)}
                                    <ObjectItem
                                      item={proc}
                                      name={proc.name}
                                      type="function"
                                      badge={proc.procedure_type === "FUNCTION"
                                        ? "F"
                                        : "P"}
                                      badgeType={proc.procedure_type ===
                                      "FUNCTION"
                                        ? "success"
                                        : "secondary"}
                                      indent={5}
                                      on:dblclick={() =>
                                        handleProcedureDblClick(
                                          proc,
                                          conn,
                                          db,
                                          schema.name
                                        )}
                                    />
                                  {/each}
                                {/if}
                              </ObjectGroup>

                              <!-- Triggers -->
                              <ObjectGroup
                                type="triggers"
                                count={getCount(schemaKey, "triggers")}
                                expanded={!!expandedGroups[
                                  getGroupKey(
                                    conn.id,
                                    db.name,
                                    "triggers",
                                    schema.name
                                  )
                                ]}
                                indent={4}
                                on:toggle={() =>
                                  handleGroupToggle(
                                    getGroupKey(
                                      conn.id,
                                      db.name,
                                      "triggers",
                                      schema.name
                                    )
                                  )}
                              >
                                {#if expandedGroups[getGroupKey(conn.id, db.name, "triggers", schema.name)]}
                                  {#each getData(schemaKey, "triggers") as trigger (`${trigger.table_name}-${trigger.name}`)}
                                    <ObjectItem
                                      item={trigger}
                                      name={trigger.name}
                                      type="trigger"
                                      indent={5}
                                    />
                                  {/each}
                                {/if}
                              </ObjectGroup>
                            {/if}
                          </SchemaItem>
                        {/each}
                      {/if}
                    </ObjectGroup>
                  {:else if conn.db_type === "MSSQL"}
                    <!-- MSSQL: Schema groups -->
                    {@const schemaGroups = groupBySchema(
                      cachedData[dbKey]?.tables || []
                    )}
                    {#each Object.entries(schemaGroups) as [schemaName, schemaTables]}
                      {@const schemaKey = `${dbKey}-${schemaName}`}
                      <SchemaItem
                        {schemaName}
                        expanded={!!expandedSchemas[schemaKey]}
                        loading={loadingSchemas[schemaKey]}
                        active={activeContextItem === `schema-${schemaKey}`}
                        indent={2}
                        icon="database"
                        on:toggle={(e) => handleSchemaToggle(e, conn, db)}
                        on:contextmenu={(e) =>
                          handleSchemaContextMenu(e, conn, db)}
                      >
                        {#if expandedSchemas[schemaKey]}
                          <!-- Tables -->
                          <ObjectGroup
                            type="tables"
                            count={schemaTables.length}
                            expanded={!!expandedGroups[
                              getGroupKey(
                                conn.id,
                                db.name,
                                "tables",
                                schemaName
                              )
                            ]}
                            indent={3}
                            on:toggle={() =>
                              handleGroupToggle(
                                getGroupKey(
                                  conn.id,
                                  db.name,
                                  "tables",
                                  schemaName
                                )
                              )}
                          >
                            {#if expandedGroups[getGroupKey(conn.id, db.name, "tables", schemaName)]}
                              {#each schemaTables as table (table.name)}
                                <ObjectItem
                                  item={table}
                                  name={table.name}
                                  type="table"
                                  size={table.size}
                                  indent={4}
                                  active={$selectedTable?.name === table.name &&
                                    $selectedTable?._connId === conn.id &&
                                    $selectedTable?._schema === schemaName}
                                  on:click={() =>
                                    handleTableClick(
                                      table,
                                      conn,
                                      db,
                                      schemaName
                                    )}
                                  on:dblclick={() =>
                                    handleTableDblClick(
                                      table,
                                      conn,
                                      db,
                                      schemaName
                                    )}
                                  on:contextmenu={(e) =>
                                    handleTableContextMenu(e, conn, db)}
                                />
                              {/each}
                            {/if}
                          </ObjectGroup>

                          <!-- Views -->
                          <ObjectGroup
                            type="views"
                            count={getCount(schemaKey, "views")}
                            expanded={!!expandedGroups[
                              getGroupKey(conn.id, db.name, "views", schemaName)
                            ]}
                            indent={3}
                            on:toggle={() =>
                              handleGroupToggle(
                                getGroupKey(
                                  conn.id,
                                  db.name,
                                  "views",
                                  schemaName
                                )
                              )}
                          >
                            {#if expandedGroups[getGroupKey(conn.id, db.name, "views", schemaName)]}
                              {#each getData(schemaKey, "views") as view (view.name)}
                                <ObjectItem
                                  item={{ ...view, schema: schemaName }}
                                  name={view.name}
                                  type="view"
                                  indent={4}
                                  on:dblclick={() =>
                                    handleViewDblClick(
                                      { ...view, schema: schemaName },
                                      conn,
                                      db,
                                      schemaName
                                    )}
                                  on:contextmenu={(e) =>
                                    handleViewContextMenu(e, conn, db)}
                                />
                              {/each}
                            {/if}
                          </ObjectGroup>

                          <!-- Indexes -->
                          <ObjectGroup
                            type="indexes"
                            count={getCount(schemaKey, "indexes")}
                            expanded={!!expandedGroups[
                              getGroupKey(
                                conn.id,
                                db.name,
                                "indexes",
                                schemaName
                              )
                            ]}
                            indent={3}
                            on:toggle={() =>
                              handleGroupToggle(
                                getGroupKey(
                                  conn.id,
                                  db.name,
                                  "indexes",
                                  schemaName
                                )
                              )}
                          >
                            {#if expandedGroups[getGroupKey(conn.id, db.name, "indexes", schemaName)]}
                              {#each getData(schemaKey, "indexes") as idx (`${idx.table_name}-${idx.name}`)}
                                <ObjectItem
                                  item={idx}
                                  name="{idx.table_name}.{idx.name}"
                                  type="index"
                                  badge={idx.is_unique ? "U" : "I"}
                                  badgeType={idx.is_unique
                                    ? "info"
                                    : "secondary"}
                                  indent={4}
                                />
                              {/each}
                            {/if}
                          </ObjectGroup>

                          <!-- Procedures/Functions -->
                          <ObjectGroup
                            type="procedures"
                            count={getCount(schemaKey, "procedures")}
                            expanded={!!expandedGroups[
                              getGroupKey(
                                conn.id,
                                db.name,
                                "procedures",
                                schemaName
                              )
                            ]}
                            indent={3}
                            on:toggle={() =>
                              handleGroupToggle(
                                getGroupKey(
                                  conn.id,
                                  db.name,
                                  "procedures",
                                  schemaName
                                )
                              )}
                          >
                            {#if expandedGroups[getGroupKey(conn.id, db.name, "procedures", schemaName)]}
                              {#each getData(schemaKey, "procedures") as proc (proc.oid || proc.name)}
                                <ObjectItem
                                  item={proc}
                                  name={proc.name}
                                  type="procedure"
                                  badge={proc.procedure_type === "FUNCTION"
                                    ? "F"
                                    : "P"}
                                  badgeType={proc.procedure_type === "FUNCTION"
                                    ? "success"
                                    : "secondary"}
                                  indent={4}
                                  on:dblclick={() =>
                                    handleProcedureDblClick(
                                      proc,
                                      conn,
                                      db,
                                      schemaName
                                    )}
                                />
                              {/each}
                            {/if}
                          </ObjectGroup>

                          <!-- Triggers -->
                          <ObjectGroup
                            type="triggers"
                            count={getCount(schemaKey, "triggers")}
                            expanded={!!expandedGroups[
                              getGroupKey(
                                conn.id,
                                db.name,
                                "triggers",
                                schemaName
                              )
                            ]}
                            indent={3}
                            on:toggle={() =>
                              handleGroupToggle(
                                getGroupKey(
                                  conn.id,
                                  db.name,
                                  "triggers",
                                  schemaName
                                )
                              )}
                          >
                            {#if expandedGroups[getGroupKey(conn.id, db.name, "triggers", schemaName)]}
                              {#each getData(schemaKey, "triggers") as trigger (`${trigger.table_name}-${trigger.name}`)}
                                <ObjectItem
                                  item={trigger}
                                  name={trigger.name}
                                  type="trigger"
                                  indent={4}
                                />
                              {/each}
                            {/if}
                          </ObjectGroup>
                        {/if}
                      </SchemaItem>
                    {/each}
                  {:else}
                    <!-- MySQL, MongoDB, Redis: Direct objects -->
                    <!-- Tables/Collections -->
                    <ObjectGroup
                      type={conn.db_type === "MongoDB"
                        ? "collections"
                        : "tables"}
                      count={getCount(dbKey, "tables")}
                      expanded={!!expandedGroups[
                        getGroupKey(conn.id, db.name, "tables")
                      ]}
                      indent={2}
                      on:toggle={() =>
                        handleGroupToggle(
                          getGroupKey(conn.id, db.name, "tables")
                        )}
                    >
                      {#if expandedGroups[getGroupKey(conn.id, db.name, "tables")]}
                        {#each getData(dbKey, "tables") as table (table.name)}
                          <ObjectItem
                            item={table}
                            name={table.name}
                            type={conn.db_type === "MongoDB"
                              ? "collection"
                              : "table"}
                            size={table.size}
                            indent={3}
                            active={$selectedTable?.name === table.name &&
                              $selectedTable?._connId === conn.id &&
                              $selectedTable?._dbName === db.name}
                            on:click={() => handleTableClick(table, conn, db)}
                            on:dblclick={() =>
                              handleTableDblClick(table, conn, db)}
                            on:contextmenu={(e) =>
                              handleTableContextMenu(e, conn, db)}
                          />
                        {/each}
                      {/if}
                    </ObjectGroup>

                    {#if conn.db_type === "MySQL"}
                      <!-- Views -->
                      <ObjectGroup
                        type="views"
                        count={getCount(dbKey, "views")}
                        expanded={!!expandedGroups[
                          getGroupKey(conn.id, db.name, "views")
                        ]}
                        indent={2}
                        on:toggle={() =>
                          handleGroupToggle(
                            getGroupKey(conn.id, db.name, "views")
                          )}
                      >
                        {#if expandedGroups[getGroupKey(conn.id, db.name, "views")]}
                          {#each getData(dbKey, "views") as view (view.name)}
                            <ObjectItem
                              item={view}
                              name={view.name}
                              type="view"
                              indent={3}
                              on:dblclick={() =>
                                handleViewDblClick(view, conn, db)}
                              on:contextmenu={(e) =>
                                handleViewContextMenu(e, conn, db)}
                            />
                          {/each}
                        {/if}
                      </ObjectGroup>

                      <!-- Indexes -->
                      <ObjectGroup
                        type="indexes"
                        count={getCount(dbKey, "indexes")}
                        expanded={!!expandedGroups[
                          getGroupKey(conn.id, db.name, "indexes")
                        ]}
                        indent={2}
                        on:toggle={() =>
                          handleGroupToggle(
                            getGroupKey(conn.id, db.name, "indexes")
                          )}
                      >
                        {#if expandedGroups[getGroupKey(conn.id, db.name, "indexes")]}
                          {#each getData(dbKey, "indexes") as idx, i (`${idx.table_name}-${idx.name}-${i}`)}
                            <ObjectItem
                              item={idx}
                              name="{idx.table_name}.{idx.name}"
                              type="index"
                              badge={idx.is_unique ? "U" : "I"}
                              badgeType={idx.is_unique ? "info" : "secondary"}
                              indent={3}
                            />
                          {/each}
                        {/if}
                      </ObjectGroup>

                      <!-- Procedures -->
                      <ObjectGroup
                        type="procedures"
                        count={getCount(dbKey, "procedures")}
                        expanded={!!expandedGroups[
                          getGroupKey(conn.id, db.name, "procedures")
                        ]}
                        indent={2}
                        on:toggle={() =>
                          handleGroupToggle(
                            getGroupKey(conn.id, db.name, "procedures")
                          )}
                      >
                        {#if expandedGroups[getGroupKey(conn.id, db.name, "procedures")]}
                          {#each getData(dbKey, "procedures") as proc (proc.oid || proc.name)}
                            <ObjectItem
                              item={proc}
                              name={proc.name}
                              type="procedure"
                              badge={proc.procedure_type === "FUNCTION"
                                ? "F"
                                : "P"}
                              badgeType={proc.procedure_type === "FUNCTION"
                                ? "success"
                                : "secondary"}
                              indent={3}
                              on:dblclick={() =>
                                handleProcedureDblClick(proc, conn, db)}
                            />
                          {/each}
                        {/if}
                      </ObjectGroup>

                      <!-- Triggers -->
                      <ObjectGroup
                        type="triggers"
                        count={getCount(dbKey, "triggers")}
                        expanded={!!expandedGroups[
                          getGroupKey(conn.id, db.name, "triggers")
                        ]}
                        indent={2}
                        on:toggle={() =>
                          handleGroupToggle(
                            getGroupKey(conn.id, db.name, "triggers")
                          )}
                      >
                        {#if expandedGroups[getGroupKey(conn.id, db.name, "triggers")]}
                          {#each getData(dbKey, "triggers") as trigger (trigger.name)}
                            <ObjectItem
                              item={trigger}
                              name={trigger.name}
                              type="trigger"
                              indent={3}
                            />
                          {/each}
                        {/if}
                      </ObjectGroup>

                      <!-- Events -->
                      <ObjectGroup
                        type="events"
                        count={getCount(dbKey, "events")}
                        expanded={!!expandedGroups[
                          getGroupKey(conn.id, db.name, "events")
                        ]}
                        indent={2}
                        on:toggle={() =>
                          handleGroupToggle(
                            getGroupKey(conn.id, db.name, "events")
                          )}
                      >
                        {#if expandedGroups[getGroupKey(conn.id, db.name, "events")]}
                          {#each getData(dbKey, "events") as event (event.name)}
                            <ObjectItem
                              item={event}
                              name={event.name}
                              type="event"
                              indent={3}
                            />
                          {/each}
                        {/if}
                      </ObjectGroup>
                    {/if}
                  {/if}
                {/if}
              </DatabaseItem>
            {/each}
          {/if}
        {/if}
      </ConnectionItem>
    {/each}
  </TreeView>

  <!-- Context Menus -->
  {#if contextMenu}
    <ConnectionContextMenu
      x={contextMenu.x}
      y={contextMenu.y}
      connection={contextMenu.connection}
      isConnected={connectedConnections[contextMenu.connection.id]}
      on:edit={handleConnectionEdit}
      on:delete={handleConnectionDelete}
      on:refresh={handleConnectionRefresh}
      on:connect={handleConnectionConnect}
      on:disconnect={handleConnectionDisconnect}
      on:copy={handleConnectionCopy}
      on:rename={handleConnectionRename}
    />
  {/if}

  {#if databaseContextMenu}
    <DatabaseContextMenu
      x={databaseContextMenu.x}
      y={databaseContextMenu.y}
      database={databaseContextMenu.database}
      connection={databaseContextMenu.connection}
      on:sqlEditor={handleDatabaseSqlEditor}
      on:viewDatabase={handleDatabaseView}
      on:copy={handleDatabaseCopy}
      on:paste={handleDatabasePaste}
      on:copyAdvancedInfo={handleDatabaseCopyAdvancedInfo}
      on:delete={handleDatabaseDelete}
      on:rename={handleDatabaseRename}
      on:refresh={handleDatabaseRefresh}
    />
  {/if}

  {#if schemaContextMenu}
    <SchemaContextMenu
      x={schemaContextMenu.x}
      y={schemaContextMenu.y}
      schema={schemaContextMenu.schema}
      database={schemaContextMenu.database}
      connection={schemaContextMenu.connection}
      on:sqlEditor={handleSchemaSqlEditor}
      on:viewSchema={handleSchemaView}
      on:viewDiagram={handleSchemaViewDiagram}
      on:importData={handleSchemaImportData}
      on:generateSql={handleSchemaGenerateSql}
      on:copy={handleSchemaCopy}
      on:paste={handleSchemaPaste}
      on:copyAdvancedInfo={handleSchemaCopyAdvancedInfo}
      on:delete={handleSchemaDelete}
      on:rename={handleSchemaRename}
      on:refresh={handleSchemaRefresh}
    />
  {/if}

  {#if tableContextMenu}
    <TableContextMenu
      x={tableContextMenu.x}
      y={tableContextMenu.y}
      table={tableContextMenu.table}
      database={tableContextMenu.database}
      connection={tableContextMenu.connection}
      on:viewTable={handleTableViewTable}
      on:viewDiagram={handleTableViewDiagram}
      on:viewData={handleTableViewData}
      on:exportData={handleTableExportData}
      on:importData={handleTableImportData}
      on:readInConsole={handleTableReadInConsole}
      on:copy={handleTableCopy}
      on:paste={handleTablePaste}
      on:copyAdvancedInfo={handleTableCopyAdvancedInfo}
      on:delete={handleTableDelete}
      on:rename={handleTableRename}
      on:refresh={handleTableRefresh}
    />
  {/if}

  {#if viewContextMenu}
    <ViewContextMenu
      x={viewContextMenu.x}
      y={viewContextMenu.y}
      view={viewContextMenu.view}
      database={viewContextMenu.database}
      connection={viewContextMenu.connection}
      on:viewStructure={handleViewStructure}
      on:viewDefinition={handleViewDefinition}
      on:viewData={handleViewData}
      on:exportData={handleViewExportData}
      on:importData={handleViewImportData}
      on:readInConsole={handleViewReadInConsole}
      on:copy={handleViewCopy}
      on:copyAdvancedInfo={handleViewCopyAdvancedInfo}
      on:rename={handleViewRename}
      on:delete={handleViewDelete}
      on:refresh={handleViewRefresh}
    />
  {/if}

  <!-- Modals -->
  {#if showConnectionModal}
    <ConnectionModal
      connection={editingConnection}
      on:save={handleSaveConnection}
      on:close={() => (showConnectionModal = false)}
    />
  {/if}

  {#if showRenameModal}
    <InputModal
      title={renameModalTitle}
      value={renameModalValue}
      on:submit={(e) => {
        if (renameModalCallback) renameModalCallback(e.detail);
        showRenameModal = false;
      }}
      on:close={() => (showRenameModal = false)}
    />
  {/if}
</div>

<style>
  .sidebar2-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
</style>
