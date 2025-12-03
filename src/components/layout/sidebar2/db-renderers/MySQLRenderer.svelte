<script>
  import { createEventDispatcher } from "svelte";
  import ObjectGroup from "../tree/ObjectGroup.svelte";
  import ObjectItem from "../tree/ObjectItem.svelte";
  import { sidebarStore } from "../../../../stores/sidebar";
  import { getGroupKey } from "../../../../services/sidebarDataService";
  import { DatabaseType } from "../../../../utils/databaseTypes";

  export let connection;
  export let database;
  export let cachedData = {};
  export let expandedGroups = {};
  export let selectedTable = null;

  const dispatch = createEventDispatcher();

  const dbKey = `${connection.id}-${database.name}`;

  function handleGroupToggle(groupKey) {
    sidebarStore.toggleGroup(groupKey);
  }

  function handleTableContextMenu(e) {
    dispatch("tableContextMenu", e.detail);
  }

  function handleViewContextMenu(e) {
    dispatch("viewContextMenu", e.detail);
  }

  function handleTableClick(e) {
    const table = e.detail?.item;
    if (!table) return;
    dispatch("tableClick", { table, database, connection, schema: null });
  }

  function handleTableDblClick(e) {
    const table = e.detail?.item;
    if (!table) return;
    dispatch("tableDblClick", { table, database, connection, schema: null });
  }

  function handleViewDblClick(e) {
    const view = e.detail?.item;
    if (!view) return;
    dispatch("viewDblClick", { view, database, connection, schema: null });
  }

  function handleProcedureDblClick(e) {
    const proc = e.detail?.item;
    if (!proc) return;
    dispatch("procedureDblClick", { proc, database, connection, schema: null });
  }

  function getData(key, type) {
    return cachedData[key]?.[type] || [];
  }

  function getCount(key, type) {
    return cachedData[key]?.[type]?.length || 0;
  }
</script>

<!-- MySQL, MongoDB, Redis: Direct objects -->

<!-- Tables/Collections -->
<ObjectGroup
  type={connection.db_type === DatabaseType.MONGODB ? "collections" : "tables"}
  count={getCount(dbKey, "tables")}
  expanded={!!expandedGroups[
    getGroupKey(connection.id, database.name, "tables")
  ]}
  indent={2}
  on:toggle={() =>
    handleGroupToggle(getGroupKey(connection.id, database.name, "tables"))}
>
  {#if expandedGroups[getGroupKey(connection.id, database.name, "tables")]}
    {#each getData(dbKey, "tables") as table (table.name)}
      <ObjectItem
        item={table}
        name={table.name}
        type={connection.db_type === DatabaseType.MONGODB
          ? "collection"
          : "table"}
        size={table.size}
        indent={3}
        active={selectedTable?.name === table.name &&
          selectedTable?._connId === connection.id &&
          selectedTable?._dbName === database.name}
        on:click={handleTableClick}
        on:dblclick={handleTableDblClick}
        on:contextmenu={handleTableContextMenu}
      />
    {/each}
  {/if}
</ObjectGroup>

{#if connection.db_type === DatabaseType.MYSQL}
  <!-- Views -->
  <ObjectGroup
    type="views"
    count={getCount(dbKey, "views")}
    expanded={!!expandedGroups[
      getGroupKey(connection.id, database.name, "views")
    ]}
    indent={2}
    on:toggle={() =>
      handleGroupToggle(getGroupKey(connection.id, database.name, "views"))}
  >
    {#if expandedGroups[getGroupKey(connection.id, database.name, "views")]}
      {#each getData(dbKey, "views") as view (view.name)}
        <ObjectItem
          item={view}
          name={view.name}
          type="view"
          indent={3}
          on:dblclick={handleViewDblClick}
          on:contextmenu={handleViewContextMenu}
        />
      {/each}
    {/if}
  </ObjectGroup>

  <!-- Indexes -->
  <ObjectGroup
    type="indexes"
    count={getCount(dbKey, "indexes")}
    expanded={!!expandedGroups[
      getGroupKey(connection.id, database.name, "indexes")
    ]}
    indent={2}
    on:toggle={() =>
      handleGroupToggle(getGroupKey(connection.id, database.name, "indexes"))}
  >
    {#if expandedGroups[getGroupKey(connection.id, database.name, "indexes")]}
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
      getGroupKey(connection.id, database.name, "procedures")
    ]}
    indent={2}
    on:toggle={() =>
      handleGroupToggle(
        getGroupKey(connection.id, database.name, "procedures")
      )}
  >
    {#if expandedGroups[getGroupKey(connection.id, database.name, "procedures")]}
      {#each getData(dbKey, "procedures") as proc (proc.oid || proc.name)}
        <ObjectItem
          item={proc}
          name={proc.name}
          type="procedure"
          badge={proc.procedure_type === "FUNCTION" ? "F" : "P"}
          badgeType={proc.procedure_type === "FUNCTION"
            ? "success"
            : "secondary"}
          indent={3}
          on:dblclick={handleProcedureDblClick}
        />
      {/each}
    {/if}
  </ObjectGroup>

  <!-- Triggers -->
  <ObjectGroup
    type="triggers"
    count={getCount(dbKey, "triggers")}
    expanded={!!expandedGroups[
      getGroupKey(connection.id, database.name, "triggers")
    ]}
    indent={2}
    on:toggle={() =>
      handleGroupToggle(getGroupKey(connection.id, database.name, "triggers"))}
  >
    {#if expandedGroups[getGroupKey(connection.id, database.name, "triggers")]}
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
      getGroupKey(connection.id, database.name, "events")
    ]}
    indent={2}
    on:toggle={() =>
      handleGroupToggle(getGroupKey(connection.id, database.name, "events"))}
  >
    {#if expandedGroups[getGroupKey(connection.id, database.name, "events")]}
      {#each getData(dbKey, "events") as event (event.name)}
        <ObjectItem item={event} name={event.name} type="event" indent={3} />
      {/each}
    {/if}
  </ObjectGroup>
{/if}
