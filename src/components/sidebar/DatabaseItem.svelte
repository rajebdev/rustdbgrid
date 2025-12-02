<script>
  import { createEventDispatcher } from "svelte";
  import ObjectSectionItem from "./ObjectSectionItem.svelte";
  import SchemaItem from "./SchemaItem.svelte";

  export let database;
  export let connection;
  export let isExpanded = false;
  export let isLoading = false;
  export let isActive = false;
  export let selectedTable = null;
  export let activeContextSchema = null;

  // For MySQL/other databases
  export let cachedTables = [];
  export let cachedViews = [];
  export let cachedIndexes = [];
  export let cachedProcedures = [];
  export let cachedTriggers = [];
  export let cachedEvents = [];
  export let expandedTables = false;
  export let expandedViews = false;
  export let expandedIndexes = false;
  export let expandedProcedures = false;
  export let expandedTriggers = false;
  export let expandedEvents = false;

  // For PostgreSQL/MSSQL - schemas
  export let schemas = [];
  export let expandedSchemasParent = false;
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
    dispatch("contextmenu", { event, database, connection });
  }

  function handleToggleTables() {
    dispatch("toggleTables");
  }

  function handleToggleViews() {
    dispatch("toggleViews");
  }

  function handleToggleIndexes() {
    dispatch("toggleIndexes");
  }

  function handleToggleProcedures() {
    dispatch("toggleProcedures");
  }

  function handleToggleTriggers() {
    dispatch("toggleTriggers");
  }

  function handleToggleEvents() {
    dispatch("toggleEvents");
  }

  function handleToggleSchemasParent() {
    dispatch("toggleSchemasParent");
  }

  function handleToggleSchema(schema) {
    dispatch("toggleSchema", { schema });
  }

  function handleSchemaContextMenu(event) {
    dispatch("schemaContextMenu", event.detail);
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

  function handleSchemaToggleTables(event) {
    const schema = event.detail?.schema || event;
    dispatch("schemaToggleTables", { schema });
  }

  function handleSchemaToggleViews(event) {
    const schema = event.detail?.schema || event;
    dispatch("schemaToggleViews", { schema });
  }

  function handleSchemaToggleIndexes(event) {
    const schema = event.detail?.schema || event;
    dispatch("schemaToggleIndexes", { schema });
  }

  function handleSchemaToggleProcedures(event) {
    const schema = event.detail?.schema || event;
    dispatch("schemaToggleProcedures", { schema });
  }

  function handleSchemaToggleTriggers(event) {
    const schema = event.detail?.schema || event;
    dispatch("schemaToggleTriggers", { schema });
  }

  $: isPostgreSQL = connection.db_type === "PostgreSQL";
  $: isMSSQL = connection.db_type === "MSSQL";
  $: isMySQL = connection.db_type === "MySQL";
  $: hasSchemas = isPostgreSQL || isMSSQL;
</script>

<div class="tree-item">
  <div class="tree-node database-node">
    <button
      class="tree-toggle"
      on:click={handleToggle}
      aria-label="Toggle database"
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
      <i class="fas fa-database tree-icon"></i>
      <span class="tree-text">{database.name}</span>
    </button>
  </div>

  {#if isExpanded}
    <div class="tree-children">
      {#if isPostgreSQL}
        <!-- PostgreSQL: Schemas Parent -->
        <div class="tree-item">
          <div class="tree-node tables-section-node">
            <button
              class="tree-toggle"
              aria-label="Toggle schemas"
              on:click={handleToggleSchemasParent}
            >
              <i
                class="fas fa-chevron-{expandedSchemasParent
                  ? 'down'
                  : 'right'}"
              ></i>
            </button>
            <button
              class="tree-section-header"
              on:click={handleToggleSchemasParent}
            >
              <i class="fas fa-folder-tree"></i>
              <span>Schemas ({schemas.length})</span>
            </button>
          </div>
          {#if expandedSchemasParent}
            <div class="tree-children">
              {#each schemas as schema}
                {@const schemaKey = `${connection.id}-${database.name}-${schema.name}`}
                <SchemaItem
                  {schema}
                  {connection}
                  {database}
                  isExpanded={expandedSchemas[schemaKey] || false}
                  isLoading={loadingSchemas[schemaKey] || false}
                  isActive={activeContextSchema === schemaKey}
                  cachedTables={cachedSchemaTables[schemaKey] || []}
                  cachedViews={cachedSchemaViews[schemaKey] || []}
                  cachedIndexes={cachedSchemaIndexes[schemaKey] || []}
                  cachedProcedures={cachedSchemaProcedures[schemaKey] || []}
                  cachedTriggers={cachedSchemaTriggers[schemaKey] || []}
                  expandedTables={expandedSchemaObjects[
                    `${schemaKey}-tables`
                  ] || false}
                  expandedViews={expandedSchemaObjects[`${schemaKey}-views`] ||
                    false}
                  expandedIndexes={expandedSchemaObjects[
                    `${schemaKey}-indexes`
                  ] || false}
                  expandedProcedures={expandedSchemaObjects[
                    `${schemaKey}-procedures`
                  ] || false}
                  expandedTriggers={expandedSchemaObjects[
                    `${schemaKey}-triggers`
                  ] || false}
                  {selectedTable}
                  on:toggle={() => handleToggleSchema(schema)}
                  on:contextmenu={handleSchemaContextMenu}
                  on:toggleTables={() => handleSchemaToggleTables(schema)}
                  on:toggleViews={() => handleSchemaToggleViews(schema)}
                  on:toggleIndexes={() => handleSchemaToggleIndexes(schema)}
                  on:toggleProcedures={() =>
                    handleSchemaToggleProcedures(schema)}
                  on:toggleTriggers={() => handleSchemaToggleTriggers(schema)}
                  on:itemSelect={handleItemSelect}
                  on:itemDoubleClick={handleItemDoubleClick}
                  on:itemContextMenu={handleItemContextMenu}
                />
              {/each}
            </div>
          {/if}
        </div>
      {:else if isMSSQL}
        <!-- MSSQL: Direct Schemas -->
        {#each schemas as schema}
          {@const schemaKey = `${connection.id}-${database.name}-${schema.name}`}
          <SchemaItem
            {schema}
            {connection}
            {database}
            isExpanded={expandedSchemas[schemaKey] || false}
            isLoading={loadingSchemas[schemaKey] || false}
            isActive={activeContextSchema === schemaKey}
            cachedTables={cachedSchemaTables[schemaKey] || []}
            cachedViews={cachedSchemaViews[schemaKey] || []}
            cachedIndexes={cachedSchemaIndexes[schemaKey] || []}
            cachedProcedures={cachedSchemaProcedures[schemaKey] || []}
            cachedTriggers={cachedSchemaTriggers[schemaKey] || []}
            expandedTables={expandedSchemaObjects[`${schemaKey}-tables`] ||
              false}
            expandedViews={expandedSchemaObjects[`${schemaKey}-views`] || false}
            expandedIndexes={expandedSchemaObjects[`${schemaKey}-indexes`] ||
              false}
            expandedProcedures={expandedSchemaObjects[
              `${schemaKey}-procedures`
            ] || false}
            expandedTriggers={expandedSchemaObjects[`${schemaKey}-triggers`] ||
              false}
            {selectedTable}
            on:toggle={() => handleToggleSchema(schema)}
            on:contextmenu={handleSchemaContextMenu}
            on:toggleTables={() => handleSchemaToggleTables(schema)}
            on:toggleViews={() => handleSchemaToggleViews(schema)}
            on:toggleIndexes={() => handleSchemaToggleIndexes(schema)}
            on:toggleProcedures={() => handleSchemaToggleProcedures(schema)}
            on:toggleTriggers={() => handleSchemaToggleTriggers(schema)}
            on:itemSelect={handleItemSelect}
            on:itemDoubleClick={handleItemDoubleClick}
            on:itemContextMenu={handleItemContextMenu}
          />
        {/each}
      {:else}
        <!-- MySQL and other databases: Direct object sections -->
        <!-- Tables -->
        <ObjectSectionItem
          sectionType="tables"
          items={cachedTables}
          isExpanded={expandedTables}
          isLoading={false}
          {connection}
          {database}
          {selectedTable}
          dbType={connection.db_type}
          on:toggle={handleToggleTables}
          on:itemSelect={handleItemSelect}
          on:itemDoubleClick={handleItemDoubleClick}
          on:itemContextMenu={handleItemContextMenu}
        />

        {#if isMySQL}
          <!-- MySQL specific objects -->
          {#if cachedViews.length > 0}
            <ObjectSectionItem
              sectionType="views"
              items={cachedViews}
              isExpanded={expandedViews}
              isLoading={false}
              {connection}
              {database}
              {selectedTable}
              dbType={connection.db_type}
              on:toggle={handleToggleViews}
              on:itemSelect={handleItemSelect}
              on:itemDoubleClick={handleItemDoubleClick}
              on:itemContextMenu={handleItemContextMenu}
            />
          {/if}

          {#if cachedIndexes.length > 0}
            <ObjectSectionItem
              sectionType="indexes"
              items={cachedIndexes}
              isExpanded={expandedIndexes}
              isLoading={false}
              {connection}
              {database}
              {selectedTable}
              dbType={connection.db_type}
              on:toggle={handleToggleIndexes}
              on:itemSelect={handleItemSelect}
              on:itemDoubleClick={handleItemDoubleClick}
              on:itemContextMenu={handleItemContextMenu}
            />
          {/if}

          {#if cachedProcedures.length > 0}
            <ObjectSectionItem
              sectionType="procedures"
              items={cachedProcedures}
              isExpanded={expandedProcedures}
              isLoading={false}
              {connection}
              {database}
              {selectedTable}
              dbType={connection.db_type}
              on:toggle={handleToggleProcedures}
              on:itemSelect={handleItemSelect}
              on:itemDoubleClick={handleItemDoubleClick}
              on:itemContextMenu={handleItemContextMenu}
            />
          {/if}

          {#if cachedTriggers.length > 0}
            <ObjectSectionItem
              sectionType="triggers"
              items={cachedTriggers}
              isExpanded={expandedTriggers}
              isLoading={false}
              {connection}
              {database}
              {selectedTable}
              dbType={connection.db_type}
              on:toggle={handleToggleTriggers}
              on:itemSelect={handleItemSelect}
              on:itemDoubleClick={handleItemDoubleClick}
              on:itemContextMenu={handleItemContextMenu}
            />
          {/if}

          {#if cachedEvents.length > 0}
            <ObjectSectionItem
              sectionType="events"
              items={cachedEvents}
              isExpanded={expandedEvents}
              isLoading={false}
              {connection}
              {database}
              {selectedTable}
              dbType={connection.db_type}
              on:toggle={handleToggleEvents}
              on:itemSelect={handleItemSelect}
              on:itemDoubleClick={handleItemDoubleClick}
              on:itemContextMenu={handleItemContextMenu}
            />
          {/if}
        {/if}
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

  .tree-label.active .tree-icon {
    color: var(--accent-blue) !important;
  }

  .tree-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tree-children {
    margin-left: 12px;
  }

  .database-node {
    padding-left: 8px;
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
</style>
