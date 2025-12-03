<script>
  import { createEventDispatcher } from "svelte";
  import TreeNode from "./TreeNode.svelte";

  export let connection = {};
  export let expanded = false;
  export let loading = false;
  export let connected = false;
  export let active = false;

  const dispatch = createEventDispatcher();

  // Database type icons mapping
  const dbIcons = {
    MySQL: { type: "emoji", value: "🐬" },
    PostgreSQL: { type: "emoji", value: "🐘" },
    MongoDB: { type: "emoji", value: "🍃" },
    Redis: { type: "icon", value: "fas fa-database", class: "redis-icon" },
    Ignite: { type: "emoji", value: "🔥" },
    MSSQL: { type: "emoji", value: "🗄️" },
    default: { type: "icon", value: "fas fa-server" },
  };

  $: iconConfig = dbIcons[connection.db_type] || dbIcons.default;

  function handleToggle() {
    dispatch("toggle", connection);
  }

  function handleClick(e) {
    dispatch("click", { event: e.detail, connection });
  }

  function handleContextMenu(e) {
    dispatch("contextmenu", { event: e.detail, connection });
  }
</script>

<TreeNode
  {expanded}
  {loading}
  {active}
  indent={0}
  on:toggle={handleToggle}
  on:click={handleClick}
  on:contextmenu={handleContextMenu}
>
  <span slot="icon" class="connection-icon-wrapper">
    {#if iconConfig.type === "emoji"}
      <span class="db-emoji">{iconConfig.value}</span>
    {:else}
      <i class="{iconConfig.value} {iconConfig.class || ''}"></i>
    {/if}
    {#if connected}
      <i class="fas fa-check-circle connection-status-badge"></i>
    {/if}
  </span>

  <span slot="label" class="connection-label">
    <span class="connection-name">{connection.name}</span>
    <span class="connection-details">
      <i>{connection.host}:{connection.port}</i>
    </span>
  </span>

  <span slot="children">
    <slot></slot>
  </span>
</TreeNode>

<style>
  .connection-icon-wrapper {
    position: relative;
    display: inline-flex;
    align-items: center;
  }

  .db-emoji {
    font-size: 14px;
    line-height: 1;
  }

  .connection-status-badge {
    position: absolute;
    bottom: -2px;
    left: -4px;
    font-size: 8px;
    color: #28a745;
  }

  .connection-label {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1;
  }

  .connection-name {
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .connection-details {
    font-size: 11px;
    color: var(--bs-secondary-color, #6c757d);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .redis-icon {
    color: #dc382d;
  }
</style>
