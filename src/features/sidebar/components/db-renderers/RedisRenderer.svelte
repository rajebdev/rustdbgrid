<script>
  import { createEventDispatcher } from "svelte";
  import ObjectGroup from "../tree/ObjectGroup.svelte";
  import ObjectItem from "../tree/ObjectItem.svelte";
  import { sidebarStore } from "../../../../stores/sidebar";
  import { getGroupKey } from "../../../../services/sidebarDataService";

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

  function handleKeyContextMenu(e) {
    dispatch("tableContextMenu", e.detail);
  }

  function handleKeyClick(e) {
    const key = e.detail?.item;
    if (!key) return;
    dispatch("tableClick", {
      table: { ...key, isRedisKey: true },
      database,
      connection,
      schema: null,
    });
  }

  function handleKeyDblClick(e) {
    const key = e.detail?.item;
    if (!key) return;
    dispatch("tableDblClick", {
      table: { ...key, isRedisKey: true },
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

  // Redis uses 'tables' key to store keys/patterns
  $: redisKeys = getData(dbKey, "tables");

  // Group keys by pattern/prefix if available
  function groupKeysByPattern(keys) {
    const grouped = {};
    keys.forEach((key) => {
      const pattern = key.pattern || key.type || "keys";
      if (!grouped[pattern]) {
        grouped[pattern] = [];
      }
      grouped[pattern].push(key);
    });
    return grouped;
  }

  $: groupedKeys = groupKeysByPattern(redisKeys);
</script>

<!-- Redis: Keys organized by type/pattern -->
{#if Object.keys(groupedKeys).length > 1}
  <!-- Multiple patterns/types - show groups -->
  {#each Object.entries(groupedKeys) as [pattern, keys]}
    <ObjectGroup
      type={pattern}
      count={keys.length}
      expanded={!!expandedGroups[
        getGroupKey(connection.id, database.name, pattern)
      ]}
      indent={2}
      on:toggle={() =>
        handleGroupToggle(getGroupKey(connection.id, database.name, pattern))}
    >
      {#if expandedGroups[getGroupKey(connection.id, database.name, pattern)]}
        {#each keys as key (key.name)}
          <ObjectItem
            item={key}
            name={key.name}
            type="key"
            badge={key.type ? key.type.substring(0, 1).toUpperCase() : "K"}
            badgeType={key.type === "string"
              ? "success"
              : key.type === "list"
                ? "info"
                : key.type === "set"
                  ? "warning"
                  : key.type === "hash"
                    ? "primary"
                    : key.type === "zset"
                      ? "secondary"
                      : "secondary"}
            size={key.size}
            indent={3}
            active={selectedTable?.name === key.name &&
              selectedTable?._connId === connection.id &&
              selectedTable?._dbName === database.name}
            on:click={handleKeyClick}
            on:dblclick={handleKeyDblClick}
            on:contextmenu={handleKeyContextMenu}
          />
        {/each}
      {/if}
    </ObjectGroup>
  {/each}
{:else}
  <!-- Single group or no patterns - show keys directly -->
  <ObjectGroup
    type="keys"
    count={redisKeys.length}
    expanded={!!expandedGroups[
      getGroupKey(connection.id, database.name, "keys")
    ]}
    indent={2}
    on:toggle={() =>
      handleGroupToggle(getGroupKey(connection.id, database.name, "keys"))}
  >
    {#if expandedGroups[getGroupKey(connection.id, database.name, "keys")]}
      {#each redisKeys as key (key.name)}
        <ObjectItem
          item={key}
          name={key.name}
          type="key"
          badge={key.type ? key.type.substring(0, 1).toUpperCase() : "K"}
          badgeType={key.type === "string"
            ? "success"
            : key.type === "list"
              ? "info"
              : key.type === "set"
                ? "warning"
                : key.type === "hash"
                  ? "primary"
                  : key.type === "zset"
                    ? "secondary"
                    : "secondary"}
          size={key.size}
          indent={3}
          active={selectedTable?.name === key.name &&
            selectedTable?._connId === connection.id &&
            selectedTable?._dbName === database.name}
          on:click={handleKeyClick}
          on:dblclick={handleKeyDblClick}
          on:contextmenu={handleKeyContextMenu}
        />
      {/each}
    {/if}
  </ObjectGroup>
{/if}
