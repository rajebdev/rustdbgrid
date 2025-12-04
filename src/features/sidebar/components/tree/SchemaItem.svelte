<script>
  import { createEventDispatcher } from "svelte";
  import TreeNode from "./TreeNode.svelte";

  export let schema = {};
  export let schemaName = "";
  export let expanded = false;
  export let loading = false;
  export let active = false;
  export let indent = 2;
  export let icon = "folder"; // folder, folder-tree, database

  const dispatch = createEventDispatcher();

  // Icon mapping
  const iconMap = {
    folder: "fas fa-folder",
    "folder-tree": "fas fa-folder-tree",
    database: "fas fa-database",
  };

  $: iconClass = iconMap[icon] || iconMap.folder;
  $: displayName = schemaName || schema.name || "Schema";

  function handleToggle() {
    dispatch("toggle", { schema, schemaName: displayName });
  }

  function handleClick(e) {
    dispatch("click", { event: e.detail, schema, schemaName: displayName });
  }

  function handleContextMenu(e) {
    dispatch("contextmenu", {
      event: e.detail,
      schema,
      schemaName: displayName,
    });
  }
</script>

<TreeNode
  {expanded}
  {loading}
  {active}
  {indent}
  on:toggle={handleToggle}
  on:click={handleClick}
  on:contextmenu={handleContextMenu}
>
  <span slot="icon">
    <i class="{iconClass} schema-icon"></i>
  </span>

  <span slot="label" class="schema-label">
    {displayName}
  </span>

  <span slot="children">
    <slot></slot>
  </span>
</TreeNode>

<style>
  .schema-icon {
    font-size: 11px;
    color: var(--accent-yellow, #ffc107);
  }

  .schema-label {
    font-weight: 400;
  }
</style>
