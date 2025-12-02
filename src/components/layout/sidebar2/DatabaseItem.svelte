<script>
  import { createEventDispatcher } from "svelte";
  import TreeNode from "./TreeNode.svelte";

  export let database = {};
  export let expanded = false;
  export let loading = false;
  export let active = false;

  const dispatch = createEventDispatcher();

  function handleToggle() {
    dispatch("toggle", database);
  }

  function handleClick(e) {
    dispatch("click", { event: e.detail, database });
  }

  function handleContextMenu(e) {
    dispatch("contextmenu", { event: e.detail, database });
  }
</script>

<TreeNode
  {expanded}
  {loading}
  {active}
  indent={1}
  on:toggle={handleToggle}
  on:click={handleClick}
  on:contextmenu={handleContextMenu}
>
  <span slot="icon">
    <i class="fas fa-database database-icon"></i>
  </span>

  <span slot="label" class="database-label">
    {database.name}
  </span>

  <span slot="children">
    <slot></slot>
  </span>
</TreeNode>

<style>
  .database-icon {
    font-size: 11px;
    color: var(--bs-secondary-color, #6c757d);
  }

  .database-label {
    font-weight: 400;
  }
</style>
