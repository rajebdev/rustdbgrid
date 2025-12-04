<script>
  import { onMount, onDestroy, createEventDispatcher } from "svelte";
  import TreeView from "./tree/TreeView.svelte";
  import ConnectionItem from "./tree/ConnectionItem.svelte";
  import DatabaseItem from "./tree/DatabaseItem.svelte";
  import ObjectItem from "./tree/ObjectItem.svelte";

  // DB Renderers
  import PostgreSQLRenderer from "./db-renderers/PostgreSQLRenderer.svelte";
  import MSSQLRenderer from "./db-renderers/MSSQLRenderer.svelte";
  import MySQLRenderer from "./db-renderers/MySQLRenderer.svelte";
  import MongoDBRenderer from "./db-renderers/MongoDBRenderer.svelte";
  import IgniteRenderer from "./db-renderers/IgniteRenderer.svelte";
  import RedisRenderer from "./db-renderers/RedisRenderer.svelte";

  // Context Menu Manager
  import ContextMenuManager from "./context-menu/ContextMenuManager.svelte";

  // Modals
  import InputModal from "../../../shared/components/modals/InputModal.svelte";
  import ConnectionModal from "../../connection/components/ConnectionModal.svelte";

  // Stores
  import {
    connections,
    activeConnection,
    selectedDatabase,
    selectedTable,
  } from "../../connection/stores/connections";
  import { sidebarStore } from "../stores/sidebar";

  // Utils
  import { getConnectionsInfo } from "../../../core/integrations/tauri";
  import { DatabaseType } from "../../../core/config/databaseTypes";

  // Services
  import {
    loadConnectionDatabases,
    loadDatabaseInfo,
    syncConnectedStatus,
    connectDatabase,
    disconnectDatabase,
    refreshConnection,
    cancelAllOperations,
  } from "../services/sidebarDataService";

  import {
    connectionHandlers,
    databaseHandlers,
    schemaHandlers,
    tableHandlers,
    viewHandlers,
  } from "../handlers/sidebarActionHandlers";

  const dispatch = createEventDispatcher();

  // Local state from store
  let state = {};
  const unsubscribe = sidebarStore.subscribe((s) => (state = s));

  // Shortcuts for reactive statements
  $: ({
    searchQuery,
    expandedConnections,
    expandedDatabases,
    expandedSchemas,
    expandedSchemasParent,
    expandedGroups,
    loadingConnections,
    loadingDatabases,
    loadingSchemas,
    connectedConnections,
    cachedData,
    activeContextItem,
    showConnectionModal,
    editingConnection,
    showRenameModal,
    renameModalData,
  } = state);

  // Lifecycle
  onMount(async () => {
    await loadConnections();
    await syncConnectedStatus();
  });

  onDestroy(() => {
    unsubscribe();
    cancelAllOperations();
  });

  async function loadConnections() {
    try {
      connections.set(await getConnectionsInfo());
    } catch (error) {
      console.error("Failed to load connections:", error);
    }
  }

  $: filteredConnections = ($connections || []).filter((conn) => {
    if (!conn?.name) return false;
    return conn.name.toLowerCase().includes((searchQuery || "").toLowerCase());
  });

  // ============================================================
  // TOGGLE HANDLERS
  // ============================================================

  async function handleConnectionToggle(e) {
    const conn = e.detail;
    if (!expandedConnections[conn.id]) {
      activeConnection.set(conn);
      try {
        await loadConnectionDatabases(conn.id);
      } catch (error) {
        console.error("Failed to load databases:", error);
      }
    } else {
      sidebarStore.toggleConnection(conn.id);
    }
  }

  async function handleDatabaseToggle(e, conn) {
    const db = e.detail;
    const key = `${conn.id}-${db.name}`;
    if (!expandedDatabases[key]) {
      const connObj = $connections.find((c) => c.id === conn.id);
      if (connObj) activeConnection.set(connObj);
      selectedDatabase.set(db);
      try {
        await loadDatabaseInfo(conn.id, db.name, conn.db_type);
      } catch (error) {
        console.error("Failed to load database:", error);
      }
    } else {
      sidebarStore.toggleDatabase(key, false);
    }
  }

  // ============================================================
  // ITEM HANDLERS
  // ============================================================

  function handleTableClick(e) {
    const { table, database, connection, schema } = e.detail;
    selectedTable.set({
      ...table,
      _connId: connection.id,
      _dbName: database.name,
      _schema: schema || table.schema || null,
    });
    const connObj = $connections.find((c) => c.id === connection.id);
    if (connObj) activeConnection.set(connObj);
    selectedDatabase.set(database);
  }

  function handleTableDblClick(e) {
    const { table, database, connection, schema } = e.detail;
    dispatch("openTableTab", {
      table: { ...table, schema: schema || table.schema || null },
      database,
      connection,
    });
  }

  function handleViewDblClick(e) {
    const { view, database, connection, schema } = e.detail;
    dispatch("openTableTab", {
      table: { name: view.name, schema: schema || view.schema, isView: true },
      database,
      connection,
    });
  }

  function handleProcedureDblClick(e) {
    const { proc, database, connection, schema } = e.detail;
    dispatch("openProcedureTab", {
      procedure: { ...proc, schema: schema || proc.schema },
      database,
      connection,
    });
  }

  // ============================================================
  // CONTEXT MENU HANDLERS
  // ============================================================

  function openContextMenu(type, event, itemId, data) {
    event.preventDefault();
    event.stopPropagation();
    sidebarStore.openContextMenu(type, {
      x: event.clientX,
      y: event.clientY,
      itemId,
      ...data,
    });
  }

  function handleConnectionContextMenu(e) {
    const { event, connection } = e.detail;
    openContextMenu("connection", event, `conn-${connection.id}`, {
      connection,
    });
  }

  function handleDatabaseContextMenu(e, conn) {
    const { event, database } = e.detail;
    openContextMenu("database", event, `db-${conn.id}-${database.name}`, {
      database,
      connection: conn,
    });
  }

  function handleSchemaContextMenu(e, conn, db) {
    const { event, schemaName } = e.detail;
    openContextMenu(
      "schema",
      event,
      `schema-${conn.id}-${db.name}-${schemaName}`,
      { schema: schemaName, database: db, connection: conn }
    );
  }

  function handleTableContextMenu(e, conn, db) {
    const { event, item } = e.detail;
    openContextMenu(
      "table",
      event,
      `table-${conn.id}-${db.name}-${item.name}`,
      {
        table: item,
        database: db,
        connection: conn,
      }
    );
  }

  function handleViewContextMenu(e, conn, db) {
    const { event, item } = e.detail;
    openContextMenu("view", event, `view-${conn.id}-${db.name}-${item.name}`, {
      view: item,
      database: db,
      connection: conn,
    });
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
      on:click={() => sidebarStore.openConnectionModal(null)}
      title="Add Connection"
    >
      <i class="fas fa-plus"></i>
    </button>

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
        {#if expandedConnections[conn.id]}
          {#if conn.db_type === DatabaseType.IGNITE}
            <IgniteRenderer
              connection={conn}
              caches={expandedConnections[conn.id].databases || []}
              selectedTable={$selectedTable}
              on:tableContextMenu={(e) =>
                handleTableContextMenu(e, conn, { name: "" })}
              on:tableClick={handleTableClick}
              on:tableDblClick={handleTableDblClick}
            />
          {:else}
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
                  {#if conn.db_type === DatabaseType.POSTGRESQL}
                    <PostgreSQLRenderer
                      connection={conn}
                      database={db}
                      {cachedData}
                      {expandedSchemasParent}
                      {expandedSchemas}
                      {expandedGroups}
                      {loadingSchemas}
                      {activeContextItem}
                      selectedTable={$selectedTable}
                      on:schemaContextMenu={(e) =>
                        handleSchemaContextMenu(e, conn, db)}
                      on:tableContextMenu={(e) =>
                        handleTableContextMenu(e, conn, db)}
                      on:viewContextMenu={(e) =>
                        handleViewContextMenu(e, conn, db)}
                      on:tableClick={handleTableClick}
                      on:tableDblClick={handleTableDblClick}
                      on:viewDblClick={handleViewDblClick}
                      on:procedureDblClick={handleProcedureDblClick}
                    />
                  {:else if conn.db_type === DatabaseType.MSSQL}
                    <MSSQLRenderer
                      connection={conn}
                      database={db}
                      {cachedData}
                      {expandedSchemas}
                      {expandedGroups}
                      {loadingSchemas}
                      {activeContextItem}
                      selectedTable={$selectedTable}
                      on:schemaContextMenu={(e) =>
                        handleSchemaContextMenu(e, conn, db)}
                      on:tableContextMenu={(e) =>
                        handleTableContextMenu(e, conn, db)}
                      on:viewContextMenu={(e) =>
                        handleViewContextMenu(e, conn, db)}
                      on:tableClick={handleTableClick}
                      on:tableDblClick={handleTableDblClick}
                      on:viewDblClick={handleViewDblClick}
                      on:procedureDblClick={handleProcedureDblClick}
                    />
                  {:else if conn.db_type === DatabaseType.MONGODB}
                    <MongoDBRenderer
                      connection={conn}
                      database={db}
                      {cachedData}
                      {expandedGroups}
                      selectedTable={$selectedTable}
                      on:tableContextMenu={(e) =>
                        handleTableContextMenu(e, conn, db)}
                      on:tableClick={handleTableClick}
                      on:tableDblClick={handleTableDblClick}
                    />
                  {:else if conn.db_type === DatabaseType.REDIS}
                    <RedisRenderer
                      connection={conn}
                      database={db}
                      {cachedData}
                      {expandedGroups}
                      selectedTable={$selectedTable}
                      on:tableContextMenu={(e) =>
                        handleTableContextMenu(e, conn, db)}
                      on:tableClick={handleTableClick}
                      on:tableDblClick={handleTableDblClick}
                    />
                  {:else}
                    <MySQLRenderer
                      connection={conn}
                      database={db}
                      {cachedData}
                      {expandedGroups}
                      selectedTable={$selectedTable}
                      on:tableContextMenu={(e) =>
                        handleTableContextMenu(e, conn, db)}
                      on:viewContextMenu={(e) =>
                        handleViewContextMenu(e, conn, db)}
                      on:tableClick={handleTableClick}
                      on:tableDblClick={handleTableDblClick}
                      on:viewDblClick={handleViewDblClick}
                      on:procedureDblClick={handleProcedureDblClick}
                    />
                  {/if}
                {/if}
              </DatabaseItem>
            {/each}
          {/if}
        {/if}
      </ConnectionItem>
    {/each}
  </TreeView>

  <ContextMenuManager
    {connectedConnections}
    onConnectionEdit={(e) =>
      connectionHandlers.edit(e.detail, getConnectionsInfo, connections)}
    onConnectionDelete={(e) =>
      connectionHandlers.delete(e.detail, getConnectionsInfo, connections)}
    onConnectionRefresh={(e) => refreshConnection(e.detail.id)}
    onConnectionConnect={async (e) => {
      await connectDatabase(e.detail.id);
      if (!expandedConnections[e.detail.id]) {
        await handleConnectionToggle({ detail: e.detail });
      }
    }}
    onConnectionDisconnect={(e) => disconnectDatabase(e.detail.id)}
    onConnectionCopy={(e) => connectionHandlers.copy(e.detail)}
    onConnectionRename={(e) =>
      connectionHandlers.rename(e.detail, getConnectionsInfo, connections)}
    onDatabaseSqlEditor={(e) =>
      databaseHandlers.sqlEditor(
        e.detail.database,
        e.detail.connection,
        dispatch
      )}
    onDatabaseView={(e) =>
      databaseHandlers.view(e.detail.database, e.detail.connection)}
    onDatabaseCopy={(e) =>
      databaseHandlers.copy(e.detail.database, e.detail.connection)}
    onDatabasePaste={(e) =>
      databaseHandlers.paste(e.detail.database, e.detail.connection)}
    onDatabaseCopyAdvancedInfo={(e) =>
      databaseHandlers.copyAdvancedInfo(e.detail.database, e.detail.connection)}
    onDatabaseDelete={(e) =>
      databaseHandlers.delete(e.detail.database, e.detail.connection)}
    onDatabaseRename={(e) =>
      databaseHandlers.rename(e.detail.database, e.detail.connection)}
    onDatabaseRefresh={(e) =>
      databaseHandlers.refresh(e.detail.database, e.detail.connection)}
    onSchemaSqlEditor={(e) =>
      schemaHandlers.sqlEditor(
        e.detail.schema,
        e.detail.database,
        e.detail.connection,
        dispatch
      )}
    onSchemaView={(e) =>
      schemaHandlers.view(
        e.detail.schema,
        e.detail.database,
        e.detail.connection
      )}
    onSchemaViewDiagram={(e) =>
      schemaHandlers.viewDiagram(
        e.detail.schema,
        e.detail.database,
        e.detail.connection
      )}
    onSchemaImportData={(e) =>
      schemaHandlers.importData(
        e.detail.schema,
        e.detail.database,
        e.detail.connection
      )}
    onSchemaGenerateSql={(e) =>
      schemaHandlers.generateSql(
        e.detail.schema,
        e.detail.database,
        e.detail.connection
      )}
    onSchemaCopy={(e) =>
      schemaHandlers.copy(
        e.detail.schema,
        e.detail.database,
        e.detail.connection
      )}
    onSchemaPaste={(e) =>
      schemaHandlers.paste(
        e.detail.schema,
        e.detail.database,
        e.detail.connection
      )}
    onSchemaCopyAdvancedInfo={(e) =>
      schemaHandlers.copyAdvancedInfo(
        e.detail.schema,
        e.detail.database,
        e.detail.connection
      )}
    onSchemaDelete={(e) =>
      schemaHandlers.delete(
        e.detail.schema,
        e.detail.database,
        e.detail.connection
      )}
    onSchemaRename={(e) =>
      schemaHandlers.rename(
        e.detail.schema,
        e.detail.database,
        e.detail.connection
      )}
    onSchemaRefresh={(e) =>
      schemaHandlers.refresh(
        e.detail.schema,
        e.detail.database,
        e.detail.connection
      )}
    onTableViewTable={(e) =>
      tableHandlers.viewTable(
        e.detail.table,
        e.detail.database,
        e.detail.connection
      )}
    onTableViewDiagram={(e) =>
      tableHandlers.viewDiagram(
        e.detail.table,
        e.detail.database,
        e.detail.connection
      )}
    onTableViewData={(e) =>
      handleTableDblClick({
        table: e.detail.table,
        database: e.detail.database,
        connection: e.detail.connection,
        schema: e.detail.table.schema,
      })}
    onTableExportData={(e) =>
      tableHandlers.exportData(
        e.detail.table,
        e.detail.database,
        e.detail.connection
      )}
    onTableImportData={(e) =>
      tableHandlers.importData(
        e.detail.table,
        e.detail.database,
        e.detail.connection
      )}
    onTableReadInConsole={(e) =>
      tableHandlers.readInConsole(
        e.detail.table,
        e.detail.database,
        e.detail.connection,
        dispatch
      )}
    onTableCopy={(e) =>
      tableHandlers.copy(
        e.detail.table,
        e.detail.database,
        e.detail.connection
      )}
    onTablePaste={(e) =>
      tableHandlers.paste(
        e.detail.table,
        e.detail.database,
        e.detail.connection
      )}
    onTableCopyAdvancedInfo={(e) =>
      tableHandlers.copyAdvancedInfo(
        e.detail.table,
        e.detail.database,
        e.detail.connection
      )}
    onTableDelete={(e) =>
      tableHandlers.delete(
        e.detail.table,
        e.detail.database,
        e.detail.connection
      )}
    onTableRename={(e) =>
      tableHandlers.rename(
        e.detail.table,
        e.detail.database,
        e.detail.connection
      )}
    onTableRefresh={(e) =>
      tableHandlers.refresh(
        e.detail.table,
        e.detail.database,
        e.detail.connection
      )}
    onViewStructure={(e) =>
      viewHandlers.structure(
        e.detail.view,
        e.detail.database,
        e.detail.connection
      )}
    onViewDefinition={(e) =>
      viewHandlers.definition(
        e.detail.view,
        e.detail.database,
        e.detail.connection
      )}
    onViewData={(e) =>
      handleViewDblClick({
        view: e.detail.view,
        database: e.detail.database,
        connection: e.detail.connection,
        schema: e.detail.view.schema,
      })}
    onViewExportData={(e) =>
      viewHandlers.exportData(
        e.detail.view,
        e.detail.database,
        e.detail.connection
      )}
    onViewImportData={(e) =>
      viewHandlers.importData(
        e.detail.view,
        e.detail.database,
        e.detail.connection
      )}
    onViewReadInConsole={(e) =>
      viewHandlers.readInConsole(
        e.detail.view,
        e.detail.database,
        e.detail.connection,
        dispatch
      )}
    onViewCopy={(e) =>
      viewHandlers.copy(e.detail.view, e.detail.database, e.detail.connection)}
    onViewCopyAdvancedInfo={(e) =>
      viewHandlers.copyAdvancedInfo(
        e.detail.view,
        e.detail.database,
        e.detail.connection
      )}
    onViewRename={(e) =>
      viewHandlers.rename(
        e.detail.view,
        e.detail.database,
        e.detail.connection
      )}
    onViewDelete={(e) =>
      viewHandlers.delete(
        e.detail.view,
        e.detail.database,
        e.detail.connection
      )}
    onViewRefresh={(e) =>
      viewHandlers.refresh(
        e.detail.view,
        e.detail.database,
        e.detail.connection
      )}
  />

  {#if showConnectionModal}
    <ConnectionModal
      connection={editingConnection}
      on:save={async () => {
        sidebarStore.closeConnectionModal();
        await loadConnections();
      }}
      on:close={() => sidebarStore.closeConnectionModal()}
    />
  {/if}

  {#if showRenameModal}
    <InputModal
      title={renameModalData?.title}
      value={renameModalData?.value}
      on:submit={(e) => {
        renameModalData?.callback?.(e.detail);
        sidebarStore.closeRenameModal();
      }}
      on:close={() => sidebarStore.closeRenameModal()}
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
