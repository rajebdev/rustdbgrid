<script>
  import { createEventDispatcher } from "svelte";
  import SchemaItem from "../tree/SchemaItem.svelte";
  import ObjectGroup from "../tree/ObjectGroup.svelte";
  import ObjectItem from "../tree/ObjectItem.svelte";
  import { sidebarStore } from "../../../../stores/sidebar";
  import {
    loadSchemaInfo,
    getGroupKey,
  } from "../../../../services/sidebarDataService";

  export let connection;
  export let database;
  export let cachedData = {};
  export let expandedSchemas = {};
  export let expandedGroups = {};
  export let loadingSchemas = {};
  export let activeContextItem = null;
  export let selectedTable = null;

  const dispatch = createEventDispatcher();

  const dbKey = `${connection.id}-${database.name}`;

  async function handleSchemaToggle(e) {
    const { schemaName } = e.detail;
    const schemaKey = `${dbKey}-${schemaName}`;
    const isExpanded = expandedSchemas[schemaKey];

    if (!isExpanded) {
      try {
        await loadSchemaInfo(connection.id, database.name, schemaName);
      } catch (error) {
        console.error("Failed to load schema:", error);
        dispatch("error", { message: `Failed to load schema: ${error}` });
      }
    } else {
      sidebarStore.toggleSchema(schemaKey, false);
    }
  }

  function handleGroupToggle(groupKey) {
    sidebarStore.toggleGroup(groupKey);
  }

  function handleSchemaContextMenu(e) {
    dispatch("schemaContextMenu", e.detail);
  }

  function handleTableContextMenu(e) {
    dispatch("tableContextMenu", e.detail);
  }

  function handleViewContextMenu(e) {
    dispatch("viewContextMenu", e.detail);
  }

  function handleTableClick(e, schemaName) {
    const table = e.detail?.item;
    if (!table) return;
    dispatch("tableClick", { table, database, connection, schema: schemaName });
  }

  function handleTableDblClick(e, schemaName) {
    const table = e.detail?.item;
    if (!table) return;
    dispatch("tableDblClick", {
      table,
      database,
      connection,
      schema: schemaName,
    });
  }

  function handleViewDblClick(e, schemaName) {
    const view = e.detail?.item;
    if (!view) return;
    dispatch("viewDblClick", {
      view,
      database,
      connection,
      schema: schemaName,
    });
  }

  function handleProcedureDblClick(e, schemaName) {
    const proc = e.detail?.item;
    if (!proc) return;
    dispatch("procedureDblClick", {
      proc,
      database,
      connection,
      schema: schemaName,
    });
  }

  function getData(key, type) {
    return cachedData[key]?.[type] || [];
  }

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

  $: schemaGroups = groupBySchema(cachedData[dbKey]?.tables || []);
</script>

<!-- MSSQL: Schema groups -->
{#each Object.entries(schemaGroups) as [schemaName, schemaTables]}
  {@const schemaKey = `${dbKey}-${schemaName}`}
  <SchemaItem
    {schemaName}
    expanded={!!expandedSchemas[schemaKey]}
    loading={loadingSchemas[schemaKey]}
    active={activeContextItem === `schema-${schemaKey}`}
    indent={2}
    icon="database"
    on:toggle={handleSchemaToggle}
    on:contextmenu={handleSchemaContextMenu}
  >
    {#if expandedSchemas[schemaKey]}
      <!-- Tables -->
      <ObjectGroup
        type="tables"
        count={schemaTables.length}
        expanded={!!expandedGroups[
          getGroupKey(connection.id, database.name, "tables", schemaName)
        ]}
        indent={3}
        on:toggle={() =>
          handleGroupToggle(
            getGroupKey(connection.id, database.name, "tables", schemaName)
          )}
      >
        {#if expandedGroups[getGroupKey(connection.id, database.name, "tables", schemaName)]}
          {#each schemaTables as table (table.name)}
            <ObjectItem
              item={table}
              name={table.name}
              type="table"
              size={table.size}
              indent={4}
              active={selectedTable?.name === table.name &&
                selectedTable?._connId === connection.id &&
                selectedTable?._schema === schemaName}
              on:click={(e) => handleTableClick(e, schemaName)}
              on:dblclick={(e) => handleTableDblClick(e, schemaName)}
              on:contextmenu={handleTableContextMenu}
            />
          {/each}
        {/if}
      </ObjectGroup>

      <!-- Views -->
      <ObjectGroup
        type="views"
        count={getCount(schemaKey, "views")}
        expanded={!!expandedGroups[
          getGroupKey(connection.id, database.name, "views", schemaName)
        ]}
        indent={3}
        on:toggle={() =>
          handleGroupToggle(
            getGroupKey(connection.id, database.name, "views", schemaName)
          )}
      >
        {#if expandedGroups[getGroupKey(connection.id, database.name, "views", schemaName)]}
          {#each getData(schemaKey, "views") as view (view.name)}
            <ObjectItem
              item={{ ...view, schema: schemaName }}
              name={view.name}
              type="view"
              indent={4}
              on:dblclick={(e) => handleViewDblClick(e, schemaName)}
              on:contextmenu={handleViewContextMenu}
            />
          {/each}
        {/if}
      </ObjectGroup>

      <!-- Indexes -->
      <ObjectGroup
        type="indexes"
        count={getCount(schemaKey, "indexes")}
        expanded={!!expandedGroups[
          getGroupKey(connection.id, database.name, "indexes", schemaName)
        ]}
        indent={3}
        on:toggle={() =>
          handleGroupToggle(
            getGroupKey(connection.id, database.name, "indexes", schemaName)
          )}
      >
        {#if expandedGroups[getGroupKey(connection.id, database.name, "indexes", schemaName)]}
          {#each getData(schemaKey, "indexes") as idx (`${idx.table_name}-${idx.name}`)}
            <ObjectItem
              item={idx}
              name="{idx.table_name}.{idx.name}"
              type="index"
              badge={idx.is_unique ? "U" : "I"}
              badgeType={idx.is_unique ? "info" : "secondary"}
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
          getGroupKey(connection.id, database.name, "procedures", schemaName)
        ]}
        indent={3}
        on:toggle={() =>
          handleGroupToggle(
            getGroupKey(connection.id, database.name, "procedures", schemaName)
          )}
      >
        {#if expandedGroups[getGroupKey(connection.id, database.name, "procedures", schemaName)]}
          {#each getData(schemaKey, "procedures") as proc (proc.oid || proc.name)}
            <ObjectItem
              item={proc}
              name={proc.name}
              type="procedure"
              badge={proc.procedure_type.toUpperCase() === "FUNCTION"
                ? "F"
                : "P"}
              badgeType={proc.procedure_type.toUpperCase() === "FUNCTION"
                ? "success"
                : "secondary"}
              indent={4}
              on:dblclick={(e) => handleProcedureDblClick(e, schemaName)}
            />
          {/each}
        {/if}
      </ObjectGroup>

      <!-- Triggers -->
      <ObjectGroup
        type="triggers"
        count={getCount(schemaKey, "triggers")}
        expanded={!!expandedGroups[
          getGroupKey(connection.id, database.name, "triggers", schemaName)
        ]}
        indent={3}
        on:toggle={() =>
          handleGroupToggle(
            getGroupKey(connection.id, database.name, "triggers", schemaName)
          )}
      >
        {#if expandedGroups[getGroupKey(connection.id, database.name, "triggers", schemaName)]}
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
