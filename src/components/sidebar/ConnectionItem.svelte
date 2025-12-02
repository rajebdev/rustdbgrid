<script>
  import { createEventDispatcher } from "svelte";
  import DatabaseItem from "./DatabaseItem.svelte";
  import TableListItem from "./TableListItem.svelte";

  export let connection;
  export let isExpanded = false;
  export let isLoading = false;
  export let isConnected = false;
  export let isActive = false;
  export let databases = [];
  export let selectedTable = null;
  export let activeContextDatabase = null;
  export let activeContextSchema = null;

  // Database expansion states
  export let expandedDatabases = {};
  export let loadingDatabases = {};

  // For MySQL/other databases
  export let cachedTables = {};
  export let cachedViews = {};
  export let cachedIndexes = {};
  export let cachedProcedures = {};
  export let cachedTriggers = {};
  export let cachedEvents = {};
  export let expandedTables = {};
  export let expandedViews = {};
  export let expandedIndexes = {};
  export let expandedProcedures = {};
  export let expandedTriggers = {};
  export let expandedEvents = {};

  // For PostgreSQL/MSSQL
  export let cachedSchemasParent = {};
  export let expandedSchemasParent = {};
  export let expandedSchemas = {};
  export let loadingSchemas = {};
  export let cachedSchemaTables = {};
  export let cachedSchemaViews = {};
  export let cachedSchemaIndexes = {};
  export let cachedSchemaProcedures = {};
  export let cachedSchemaTriggers = {};
  export let expandedSchemaObjects = {};

  const dispatch = createEventDispatcher();

  function handleToggle() {
    dispatch("toggle");
  }

  function handleContextMenu(event) {
    dispatch("contextmenu", { event, connection });
  }

  function handleDatabaseToggle(db) {
    dispatch("databaseToggle", { database: db });
  }

  function handleDatabaseContextMenu(event) {
    dispatch("databaseContextMenu", event.detail);
  }

  function handleToggleTables(event) {
    dispatch("toggleTables", event.detail);
  }

  function handleToggleViews(event) {
    dispatch("toggleViews", event.detail);
  }

  function handleToggleIndexes(event) {
    dispatch("toggleIndexes", event.detail);
  }

  function handleToggleProcedures(event) {
    dispatch("toggleProcedures", event.detail);
  }

  function handleToggleTriggers(event) {
    dispatch("toggleTriggers", event.detail);
  }

  function handleToggleEvents(event) {
    dispatch("toggleEvents", event.detail);
  }

  function handleToggleSchemasParent(event) {
    dispatch("toggleSchemasParent", event.detail);
  }

  function handleToggleSchema(event) {
    dispatch("toggleSchema", event.detail);
  }

  function handleSchemaContextMenu(event) {
    dispatch("schemaContextMenu", event.detail);
  }

  function handleSchemaToggleTables(event) {
    dispatch("schemaToggleTables", event.detail);
  }

  function handleSchemaToggleViews(event) {
    dispatch("schemaToggleViews", event.detail);
  }

  function handleSchemaToggleIndexes(event) {
    dispatch("schemaToggleIndexes", event.detail);
  }

  function handleSchemaToggleProcedures(event) {
    dispatch("schemaToggleProcedures", event.detail);
  }

  function handleSchemaToggleTriggers(event) {
    dispatch("schemaToggleTriggers", event.detail);
  }

  function handleItemSelect(event) {
    dispatch("itemSelect", event.detail);
  }

  function handleItemDoubleClick(event) {
    dispatch("itemDoubleClick", event.detail);
  }

  function handleItemContextMenu(event) {
    dispatch("itemContextMenu", event.detail);
  }

  // Get icon based on database type
  function getIcon() {
    switch (connection.db_type) {
      case "MySQL":
        return "🐬";
      case "PostgreSQL":
        return "🐘";
      case "MongoDB":
        return "🍃";
      case "Redis":
        return null; // Will use fa-database with redis-icon class
      case "Ignite":
        return "🔥";
      case "MSSQL":
        return "🗄️";
      default:
        return null; // Will use fa-server
    }
  }

  $: icon = getIcon();
  $: isIgnite = connection.db_type === "Ignite";
</script>

<div class="tree-item">
  <div class="tree-node">
    <button
      class="tree-toggle"
      on:click={handleToggle}
      aria-label="Toggle connection"
    >
      {#if isLoading}
        <i class="fas fa-spinner fa-spin"></i>
      {:else}
        <i class="fas fa-chevron-{isExpanded ? 'down' : 'right'}"></i>
      {/if}
    </button>
    <button
      class="tree-label"
      class:active={isActive}
      on:click={handleToggle}
      on:contextmenu={handleContextMenu}
    >
      <div class="connection-icon-wrapper">
        {#if icon}
          <span class="tree-icon connection-icon db-emoji">{icon}</span>
        {:else if connection.db_type === "Redis"}
          <i class="fas fa-database tree-icon connection-icon redis-icon"></i>
        {:else}
          <i class="fas fa-server tree-icon connection-icon"></i>
        {/if}
        {#if isConnected}
          <i class="fas fa-check-circle connection-status-badge"></i>
        {/if}
      </div>
      <span class="tree-text">
        {connection.name}
      </span>
      <span class="connection-details">
        <i>{connection.host}:{connection.port}</i>
      </span>
      <span class="tree-badge">
        <!-- {connection.db_type} -->
      </span>
    </button>
  </div>

  {#if isExpanded}
    <div class="tree-children">
      {#if isIgnite}
        <!-- Ignite: Direct Caches (no database level) -->
        <table
          class="table table-sm table-hover mb-0 table-borderless"
          style="padding-left: 8px;"
        >
          <tbody>
            {#each databases as cache (cache.name)}
              <TableListItem
                table={{ name: cache.name, schema: null, size: null }}
                {connection}
                database={{ name: cache.name }}
                dbType={connection.db_type}
                isActive={selectedTable?.name === cache.name &&
                  selectedTable?._connId === connection.id}
                on:select={(e) =>
                  handleItemSelect({
                    detail: { ...e.detail, database: { name: cache.name } },
                  })}
                on:doubleclick={(e) =>
                  handleItemDoubleClick({
                    detail: { ...e.detail, database: { name: cache.name } },
                  })}
                on:contextmenu={(e) =>
                  handleItemContextMenu({
                    detail: { ...e.detail, database: { name: cache.name } },
                  })}
              />
            {/each}
          </tbody>
        </table>
      {:else}
        <!-- Other databases: normal hierarchy -->
        {#each databases as db (db.name)}
          {@const dbKey = `${connection.id}-${db.name}`}
          {@const dbData = expandedDatabases[dbKey]}
          <DatabaseItem
            database={db}
            {connection}
            isExpanded={!!dbData}
            isLoading={loadingDatabases[dbKey] || false}
            isActive={activeContextDatabase === dbKey}
            {selectedTable}
            {activeContextSchema}
            cachedTables={cachedTables[dbKey] || []}
            cachedViews={cachedViews[dbKey] || []}
            cachedIndexes={cachedIndexes[dbKey] || []}
            cachedProcedures={cachedProcedures[dbKey] || []}
            cachedTriggers={cachedTriggers[dbKey] || []}
            cachedEvents={cachedEvents[dbKey] || []}
            expandedTables={expandedTables[dbKey] || false}
            expandedViews={expandedViews[dbKey] || false}
            expandedIndexes={expandedIndexes[dbKey] || false}
            expandedProcedures={expandedProcedures[dbKey] || false}
            expandedTriggers={expandedTriggers[dbKey] || false}
            expandedEvents={expandedEvents[dbKey] || false}
            schemas={cachedSchemasParent[dbKey] || dbData?.schemas || []}
            expandedSchemasParent={expandedSchemasParent[dbKey] || false}
            {expandedSchemas}
            {loadingSchemas}
            {cachedSchemaTables}
            {cachedSchemaViews}
            {cachedSchemaIndexes}
            {cachedSchemaProcedures}
            {cachedSchemaTriggers}
            {expandedSchemaObjects}
            on:toggle={() => handleDatabaseToggle(db)}
            on:contextmenu={handleDatabaseContextMenu}
            on:toggleTables={() => handleToggleTables({ database: db })}
            on:toggleViews={() => handleToggleViews({ database: db })}
            on:toggleIndexes={() => handleToggleIndexes({ database: db })}
            on:toggleProcedures={() => handleToggleProcedures({ database: db })}
            on:toggleTriggers={() => handleToggleTriggers({ database: db })}
            on:toggleEvents={() => handleToggleEvents({ database: db })}
            on:toggleSchemasParent={() =>
              handleToggleSchemasParent({ database: db })}
            on:toggleSchema={(e) =>
              handleToggleSchema({ database: db, ...e.detail })}
            on:schemaContextMenu={handleSchemaContextMenu}
            on:schemaToggleTables={(e) =>
              handleSchemaToggleTables({ database: db, ...e.detail })}
            on:schemaToggleViews={(e) =>
              handleSchemaToggleViews({ database: db, ...e.detail })}
            on:schemaToggleIndexes={(e) =>
              handleSchemaToggleIndexes({ database: db, ...e.detail })}
            on:schemaToggleProcedures={(e) =>
              handleSchemaToggleProcedures({ database: db, ...e.detail })}
            on:schemaToggleTriggers={(e) =>
              handleSchemaToggleTriggers({ database: db, ...e.detail })}
            on:itemSelect={handleItemSelect}
            on:itemDoubleClick={handleItemDoubleClick}
            on:itemContextMenu={handleItemContextMenu}
          />
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
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
</style>
