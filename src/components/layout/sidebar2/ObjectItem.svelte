<script>
  import { createEventDispatcher } from "svelte";

  export let item = {};
  export let type = "table"; // table, view, index, procedure, trigger, event, collection, cache
  export let name = "";
  export let size = null;
  export let badge = null; // U for unique, I for index, F for function, P for procedure
  export let badgeType = "secondary"; // secondary, info, success, warning
  export let active = false;
  export let indent = 3;

  const dispatch = createEventDispatcher();

  // Type configuration
  const typeConfig = {
    table: { icon: "fas fa-table" },
    view: { icon: "fas fa-eye" },
    index: { icon: "fas fa-key" },
    procedure: { icon: "fas fa-cog" },
    function: { icon: "fas fa-cog" },
    trigger: { icon: "fas fa-bolt" },
    event: { icon: "fas fa-calendar" },
    collection: { icon: "fas fa-layer-group" },
    cache: { icon: "fas fa-server" },
  };

  $: config = typeConfig[type] || typeConfig.table;
  $: displayName = name || item.name || "Item";

  function handleClick(e) {
    dispatch("click", { event: e, item });
  }

  function handleDblClick(e) {
    dispatch("dblclick", { event: e, item });
  }

  function handleContextMenu(e) {
    e.preventDefault();
    dispatch("contextmenu", { event: e, item });
  }
</script>

<!-- svelte-ignore a11y-role-has-required-aria-props -->
<!-- svelte-ignore a11y-click-events-have-key-events -->
<div
  class="object-item"
  class:active
  style="--indent: {indent}"
  on:click={handleClick}
  on:dblclick={handleDblClick}
  on:contextmenu={handleContextMenu}
  role="treeitem"
  tabindex="0"
>
  <span class="item-toggle-placeholder"></span>

  <i class="{config.icon} item-icon"></i>

  <span class="item-name" title={displayName}>
    {displayName}
  </span>

  {#if size !== null && size !== undefined}
    <span class="item-size" title="Size">
      {size}
    </span>
  {/if}

  {#if badge}
    <span
      class="item-badge badge bg-{badgeType}"
      title={badge === "U"
        ? "Unique"
        : badge === "I"
          ? "Index"
          : badge === "F"
            ? "Function"
            : badge === "P"
              ? "Procedure"
              : ""}
    >
      {badge}
    </span>
  {/if}
</div>

<style>
  .object-item {
    --node-indent: calc(var(--indent, 0) * 12px);
    display: flex;
    align-items: center;
    padding: 2px 8px 2px calc(4px + var(--node-indent));
    cursor: pointer;
    border-radius: 4px;
    gap: 4px;
    min-height: 22px;
    transition: background-color 0.15s ease;
  }

  .object-item:hover {
    background-color: var(--bs-tertiary-bg, rgba(0, 0, 0, 0.05));
  }

  .object-item.active {
    background-color: var(--bs-primary-bg-subtle, rgba(13, 110, 253, 0.1));
  }

  .item-toggle-placeholder {
    width: 16px;
    flex-shrink: 0;
  }

  .item-icon {
    font-size: 11px;
    color: var(--bs-secondary-color, #6c757d);
    flex-shrink: 0;
  }

  .item-name {
    flex: 1;
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .item-size {
    font-size: 10px;
    color: var(--bs-secondary-color, #6c757d);
    background-color: var(--bs-tertiary-bg, #f8f9fa);
    padding: 0 4px;
    border-radius: 3px;
    flex-shrink: 0;
  }

  .item-badge {
    font-size: 9px;
    padding: 0 4px;
    min-width: 16px;
    text-align: center;
    flex-shrink: 0;
  }
</style>
