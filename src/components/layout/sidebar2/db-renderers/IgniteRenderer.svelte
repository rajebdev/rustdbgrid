<script>
  import { createEventDispatcher } from "svelte";
  import ObjectItem from "../tree/ObjectItem.svelte";

  export let connection;
  export let caches = [];
  export let selectedTable = null;

  const dispatch = createEventDispatcher();

  function handleCacheContextMenu(e) {
    dispatch("tableContextMenu", e.detail);
  }

  function handleCacheClick(e) {
    const cache = e.detail?.item;
    if (!cache) return;
    dispatch("tableClick", {
      table: { name: cache.name, schema: null, isCache: true },
      database: { name: cache.name },
      connection,
      schema: null,
    });
  }

  function handleCacheDblClick(e) {
    const cache = e.detail?.item;
    if (!cache) return;
    dispatch("tableDblClick", {
      table: { name: cache.name, schema: null, isCache: true },
      database: { name: cache.name },
      connection,
      schema: null,
    });
  }
</script>

<!-- Apache Ignite: Direct caches (no database structure) -->
{#each caches as cache (cache.name)}
  <ObjectItem
    item={cache}
    name={cache.name}
    type="cache"
    size={cache.size}
    indent={1}
    active={selectedTable?.name === cache.name &&
      selectedTable?._connId === connection.id}
    on:click={handleCacheClick}
    on:dblclick={handleCacheDblClick}
    on:contextmenu={handleCacheContextMenu}
  />
{/each}
