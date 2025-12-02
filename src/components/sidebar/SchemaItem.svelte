<script>
  import { createEventDispatcher } from "svelte";
  import ObjectSectionItem from "./ObjectSectionItem.svelte";

  export let schema;
  export let connection;
  export let database;
  export let isExpanded = false;
  export let isLoading = false;
  export let isActive = false;
  export let cachedTables = [];
  export let cachedViews = [];
  export let cachedIndexes = [];
  export let cachedProcedures = [];
  export let cachedTriggers = [];
  export let expandedTables = false;
  export let expandedViews = false;
  export let expandedIndexes = false;
  export let expandedProcedures = false;
  export let expandedTriggers = false;
  export let selectedTable = null;

  const dispatch = createEventDispatcher();

  function handleToggle() {
    dispatch("toggle");
  }

  function handleContextMenu(event) {
    dispatch("contextmenu", { event, schema, database, connection });
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

  function handleItemSelect(event) {
    dispatch("itemSelect", event.detail);
  }

  function handleItemDoubleClick(event) {
    dispatch("itemDoubleClick", event.detail);
  }

  function handleItemContextMenu(event) {
    dispatch("itemContextMenu", event.detail);
  }
</script>

<div class="tree-item">
  <div class="tree-node tables-section-node">
    <button
      class="tree-toggle"
      aria-label="Toggle schema"
      on:click={handleToggle}
    >
      {#if isLoading}
        <i class="fas fa-spinner fa-spin"></i>
      {:else}
        <i class="fas fa-chevron-{isExpanded ? 'down' : 'right'}"></i>
      {/if}
    </button>
    <button
      class="tree-section-header"
      class:active={isActive}
      on:click={handleToggle}
      on:contextmenu={handleContextMenu}
    >
      <i class="fas fa-folder"></i>
      <span>{schema.name}</span>
    </button>
  </div>
  {#if isExpanded}
    <div class="tree-children">
      <!-- Tables -->
      <ObjectSectionItem
        sectionType="tables"
        items={cachedTables}
        isExpanded={expandedTables}
        isLoading={false}
        {connection}
        {database}
        schema={schema.name}
        {selectedTable}
        dbType={connection.db_type}
        on:toggle={handleToggleTables}
        on:itemSelect={handleItemSelect}
        on:itemDoubleClick={handleItemDoubleClick}
        on:itemContextMenu={handleItemContextMenu}
      />

      <!-- Views -->
      {#if cachedViews.length > 0}
        <ObjectSectionItem
          sectionType="views"
          items={cachedViews}
          isExpanded={expandedViews}
          isLoading={false}
          {connection}
          {database}
          schema={schema.name}
          {selectedTable}
          dbType={connection.db_type}
          on:toggle={handleToggleViews}
          on:itemSelect={handleItemSelect}
          on:itemDoubleClick={handleItemDoubleClick}
          on:itemContextMenu={handleItemContextMenu}
        />
      {/if}

      <!-- Indexes -->
      {#if cachedIndexes.length > 0}
        <ObjectSectionItem
          sectionType="indexes"
          items={cachedIndexes}
          isExpanded={expandedIndexes}
          isLoading={false}
          {connection}
          {database}
          schema={schema.name}
          {selectedTable}
          dbType={connection.db_type}
          on:toggle={handleToggleIndexes}
          on:itemSelect={handleItemSelect}
          on:itemDoubleClick={handleItemDoubleClick}
          on:itemContextMenu={handleItemContextMenu}
        />
      {/if}

      <!-- Procedures -->
      {#if cachedProcedures.length > 0}
        <ObjectSectionItem
          sectionType="procedures"
          items={cachedProcedures}
          isExpanded={expandedProcedures}
          isLoading={false}
          {connection}
          {database}
          schema={schema.name}
          {selectedTable}
          dbType={connection.db_type}
          on:toggle={handleToggleProcedures}
          on:itemSelect={handleItemSelect}
          on:itemDoubleClick={handleItemDoubleClick}
          on:itemContextMenu={handleItemContextMenu}
        />
      {/if}

      <!-- Triggers -->
      {#if cachedTriggers.length > 0}
        <ObjectSectionItem
          sectionType="triggers"
          items={cachedTriggers}
          isExpanded={expandedTriggers}
          isLoading={false}
          {connection}
          {database}
          schema={schema.name}
          {selectedTable}
          dbType={connection.db_type}
          on:toggle={handleToggleTriggers}
          on:itemSelect={handleItemSelect}
          on:itemDoubleClick={handleItemDoubleClick}
          on:itemContextMenu={handleItemContextMenu}
        />
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
