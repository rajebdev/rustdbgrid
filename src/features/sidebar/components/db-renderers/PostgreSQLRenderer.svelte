<script>
  import { createEventDispatcher } from "svelte";
  import SchemaItem from "../tree/SchemaItem.svelte";
  import ObjectGroup from "../tree/ObjectGroup.svelte";
  import ObjectItem from "../tree/ObjectItem.svelte";
  import { sidebarStore } from "../../stores/sidebar";
  import {
    loadSchemaInfo,
    getGroupKey,
  } from "../../services/sidebarDataService";

  export let connection;
  export let database;
  export let cachedData = {};
  export let expandedSchemasParent = {};
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

  function handleSchemasParentToggle() {
    sidebarStore.toggleSchemasParent(dbKey);
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

  function handleTableClick(e, schema) {
    const table = e.detail?.item;
    if (!table) return;
    dispatch("tableClick", { table, database, connection, schema });
  }

  function handleTableDblClick(e, schema) {
    const table = e.detail?.item;
    if (!table) return;
    dispatch("tableDblClick", { table, database, connection, schema });
  }

  function handleViewDblClick(e, schema) {
    const view = e.detail?.item;
    if (!view) return;
    dispatch("viewDblClick", { view, database, connection, schema });
  }

  function handleProcedureDblClick(e, schema) {
    const proc = e.detail?.item;
    if (!proc) return;
    dispatch("procedureDblClick", { proc, database, connection, schema });
  }

  function getData(key, type) {
    return cachedData[key]?.[type] || [];
  }

  function getCount(key, type) {
    return cachedData[key]?.[type]?.length || 0;
  }
</script>

<!-- PostgreSQL: Schemas parent -->
<ObjectGroup
  type="schemas"
  count={cachedData[dbKey]?.schemas?.length || 0}
  expanded={!!expandedSchemasParent[dbKey]}
  indent={2}
  on:toggle={handleSchemasParentToggle}
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
        on:toggle={handleSchemaToggle}
        on:contextmenu={handleSchemaContextMenu}
      >
        {#if expandedSchemas[schemaKey]}
          <!-- Tables -->
          <ObjectGroup
            type="tables"
            count={getCount(schemaKey, "tables")}
            expanded={!!expandedGroups[
              getGroupKey(connection.id, database.name, "tables", schema.name)
            ]}
            indent={4}
            on:toggle={() =>
              handleGroupToggle(
                getGroupKey(connection.id, database.name, "tables", schema.name)
              )}
          >
            {#if expandedGroups[getGroupKey(connection.id, database.name, "tables", schema.name)]}
              {#each getData(schemaKey, "tables") as table (table.name)}
                <ObjectItem
                  item={table}
                  name={table.name}
                  type="table"
                  size={table.size}
                  indent={5}
                  active={selectedTable?.name === table.name &&
                    selectedTable?._connId === connection.id &&
                    selectedTable?._schema === schema.name}
                  on:click={(e) => handleTableClick(e, schema.name)}
                  on:dblclick={(e) => handleTableDblClick(e, schema.name)}
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
              getGroupKey(connection.id, database.name, "views", schema.name)
            ]}
            indent={4}
            on:toggle={() =>
              handleGroupToggle(
                getGroupKey(connection.id, database.name, "views", schema.name)
              )}
          >
            {#if expandedGroups[getGroupKey(connection.id, database.name, "views", schema.name)]}
              {#each getData(schemaKey, "views") as view (view.name)}
                <ObjectItem
                  item={view}
                  name={view.name}
                  type="view"
                  indent={5}
                  on:dblclick={(e) => handleViewDblClick(e, schema.name)}
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
              getGroupKey(connection.id, database.name, "indexes", schema.name)
            ]}
            indent={4}
            on:toggle={() =>
              handleGroupToggle(
                getGroupKey(
                  connection.id,
                  database.name,
                  "indexes",
                  schema.name
                )
              )}
          >
            {#if expandedGroups[getGroupKey(connection.id, database.name, "indexes", schema.name)]}
              {#each getData(schemaKey, "indexes") as idx (`${idx.table_name}-${idx.name}`)}
                <ObjectItem
                  item={idx}
                  name="{idx.table_name}.{idx.name}"
                  type="index"
                  badge={idx.is_unique ? "U" : "I"}
                  badgeType={idx.is_unique ? "info" : "secondary"}
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
                connection.id,
                database.name,
                "functions",
                schema.name
              )
            ]}
            indent={4}
            on:toggle={() =>
              handleGroupToggle(
                getGroupKey(
                  connection.id,
                  database.name,
                  "functions",
                  schema.name
                )
              )}
          >
            {#if expandedGroups[getGroupKey(connection.id, database.name, "functions", schema.name)]}
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
                  indent={5}
                  on:dblclick={(e) => handleProcedureDblClick(e, schema.name)}
                />
              {/each}
            {/if}
          </ObjectGroup>

          <!-- Triggers -->
          <ObjectGroup
            type="triggers"
            count={getCount(schemaKey, "triggers")}
            expanded={!!expandedGroups[
              getGroupKey(connection.id, database.name, "triggers", schema.name)
            ]}
            indent={4}
            on:toggle={() =>
              handleGroupToggle(
                getGroupKey(
                  connection.id,
                  database.name,
                  "triggers",
                  schema.name
                )
              )}
          >
            {#if expandedGroups[getGroupKey(connection.id, database.name, "triggers", schema.name)]}
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
