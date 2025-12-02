<script>
  import { createEventDispatcher } from "svelte";
  import TableListItem from "./TableListItem.svelte";
  import MySQLObjectListItem from "./MySQLObjectListItem.svelte";

  export let sectionType = "tables"; // tables, views, indexes, procedures, triggers, events, schemas
  export let items = [];
  export let isExpanded = false;
  export let isLoading = false;
  export let connection;
  export let database;
  export let schema = null;
  export let selectedItem = null;
  export let dbType = "MySQL";

  const dispatch = createEventDispatcher();

  function handleToggle() {
    dispatch("toggle");
  }

  function getIcon() {
    switch (sectionType) {
      case "tables":
        return dbType === "MongoDB"
          ? "fa-layer-group"
          : dbType === "Redis" || dbType === "Ignite"
            ? "fa-server"
            : "fa-table";
      case "views":
        return "fa-eye";
      case "indexes":
        return "fa-sort-alpha-down";
      case "procedures":
        return "fa-code";
      case "triggers":
        return "fa-bolt";
      case "events":
        return "fa-calendar-alt";
      case "schemas":
        return "fa-folder-tree";
      default:
        return "fa-folder";
    }
  }

  function getLabel() {
    switch (sectionType) {
      case "tables":
        return dbType === "MongoDB"
          ? "Collections"
          : dbType === "Redis"
            ? "Keys"
            : dbType === "Ignite"
              ? "Caches"
              : "Tables";
      case "views":
        return "Views";
      case "indexes":
        return "Indexes";
      case "procedures":
        return "Procedures";
      case "triggers":
        return "Triggers";
      case "events":
        return "Events";
      case "schemas":
        return "Schemas";
      default:
        return "Items";
    }
  }

  function isItemActive(item) {
    if (!selectedItem) return false;
    return selectedItem.name === item.name;
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

  $: isMySQLObject =
    sectionType !== "tables" && dbType === "MySQL" && schema === null;
  $: isTableSection = sectionType === "tables";
</script>

<div class="tree-item">
  <div class="tree-node tables-section-node">
    <button
      class="tree-toggle"
      aria-label="Toggle {getLabel()}"
      on:click={handleToggle}
    >
      {#if isLoading}
        <i class="fas fa-spinner fa-spin"></i>
      {:else}
        <i class="fas fa-chevron-{isExpanded ? 'down' : 'right'}"></i>
      {/if}
    </button>
    <button class="tree-section-header" on:click={handleToggle}>
      <i class="fas {getIcon()}"></i>
      <span>{getLabel()} ({items.length})</span>
    </button>
  </div>
  {#if isExpanded}
    <div class="tree-children">
      <table
        class="table table-sm table-hover mb-0 table-borderless"
        class:mysql-object-table={isMySQLObject}
        style="padding-left: 8px;"
      >
        <tbody>
          {#each items as item (item.name)}
            {#if isMySQLObject}
              <MySQLObjectListItem
                {item}
                {connection}
                {database}
                {schema}
                objectType={sectionType.slice(0, -1)}
                on:select={handleItemSelect}
                on:doubleclick={handleItemDoubleClick}
                on:contextmenu={handleItemContextMenu}
              />
            {:else if isTableSection}
              <TableListItem
                table={item}
                {connection}
                {database}
                {dbType}
                isActive={isItemActive(item)}
                on:select={handleItemSelect}
                on:doubleclick={handleItemDoubleClick}
                on:contextmenu={handleItemContextMenu}
              />
            {:else}
              <!-- PostgreSQL/MSSQL schema objects use TableListItem -->
              <TableListItem
                table={item}
                {connection}
                {database}
                {dbType}
                isActive={isItemActive(item)}
                on:select={handleItemSelect}
                on:doubleclick={handleItemDoubleClick}
                on:contextmenu={handleItemContextMenu}
              />
            {/if}
          {/each}
        </tbody>
      </table>
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

  /* MySQL Object Items Styling */
  .mysql-object-table {
    background: transparent !important;
  }
</style>
