<script>
  import { createEventDispatcher } from "svelte";
  import ObjectGroup from "../tree/ObjectGroup.svelte";
  import ObjectItem from "../tree/ObjectItem.svelte";
  import { sidebarStore } from "../../stores/sidebar";
  import { getGroupKey } from "../../services/sidebarDataService";

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

  function handleCollectionContextMenu(e) {
    dispatch("tableContextMenu", e.detail);
  }

  function handleCollectionClick(e) {
    const collection = e.detail?.item;
    if (!collection) return;
    dispatch("tableClick", {
      table: { ...collection, isCollection: true },
      database,
      connection,
      schema: null,
    });
  }

  function handleCollectionDblClick(e) {
    const collection = e.detail?.item;
    if (!collection) return;
    dispatch("tableDblClick", {
      table: { ...collection, isCollection: true },
      database,
      connection,
      schema: null,
    });
  }

  function getData(key, type) {
    return cachedData[key]?.[type] || [];
  }

  function getCount(key, type) {
    return cachedData[key]?.[type]?.length || 0;
  }

  // Get collections (MongoDB uses 'tables' key in backend)
  $: collections = getData(dbKey, "tables");
</script>

<!-- MongoDB: Collections -->
<ObjectGroup
  type="collections"
  count={collections.length}
  expanded={!!expandedGroups[
    getGroupKey(connection.id, database.name, "collections")
  ]}
  indent={2}
  on:toggle={() =>
    handleGroupToggle(getGroupKey(connection.id, database.name, "collections"))}
>
  {#if expandedGroups[getGroupKey(connection.id, database.name, "collections")]}
    {#each collections as collection (collection.name)}
      <ObjectItem
        item={collection}
        name={collection.name}
        type="collection"
        size={collection.size}
        indent={3}
        active={selectedTable?.name === collection.name &&
          selectedTable?._connId === connection.id &&
          selectedTable?._dbName === database.name}
        on:click={handleCollectionClick}
        on:dblclick={handleCollectionDblClick}
        on:contextmenu={handleCollectionContextMenu}
      />
    {/each}
  {/if}
</ObjectGroup>

<!-- MongoDB: Indexes (optional, if MongoDB indexes are separate) -->
{#if getData(dbKey, "indexes").length > 0}
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
      {#each getData(dbKey, "indexes") as idx, i (`${idx.collection_name || idx.table_name}-${idx.name}-${i}`)}
        <ObjectItem
          item={idx}
          name="{idx.collection_name || idx.table_name}.{idx.name}"
          type="index"
          badge={idx.is_unique ? "U" : "I"}
          badgeType={idx.is_unique ? "info" : "secondary"}
          indent={3}
        />
      {/each}
    {/if}
  </ObjectGroup>
{/if}
