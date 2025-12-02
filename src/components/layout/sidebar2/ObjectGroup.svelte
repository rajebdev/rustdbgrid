<script>
  import { createEventDispatcher } from "svelte";
  import TreeNode from "./TreeNode.svelte";

  export let type = "tables"; // tables, views, indexes, procedures, triggers, events, collections, caches, schemas
  export let count = 0;
  export let expanded = false;
  export let loading = false;
  export let indent = 2;

  const dispatch = createEventDispatcher();

  // Type configuration
  const typeConfig = {
    tables: { icon: "fas fa-table", label: "Tables" },
    views: { icon: "fas fa-eye", label: "Views" },
    indexes: { icon: "fas fa-key", label: "Indexes" },
    procedures: { icon: "fas fa-cog", label: "Procedures" },
    functions: { icon: "fas fa-cog", label: "Functions" },
    triggers: { icon: "fas fa-bolt", label: "Triggers" },
    events: { icon: "fas fa-calendar", label: "Events" },
    collections: { icon: "fas fa-layer-group", label: "Collections" },
    caches: { icon: "fas fa-server", label: "Caches" },
    schemas: { icon: "fas fa-folder-tree", label: "Schemas" },
  };

  $: config = typeConfig[type] || typeConfig.tables;

  function handleToggle() {
    dispatch("toggle", { type });
  }
</script>

<TreeNode
  {expanded}
  {loading}
  {indent}
  selectable={true}
  on:toggle={handleToggle}
>
  <span slot="icon">
    <i class="{config.icon} group-icon"></i>
  </span>

  <span slot="label" class="group-label">
    {config.label} ({count})
  </span>

  <span slot="children">
    <slot></slot>
  </span>
</TreeNode>

<style>
  .group-icon {
    font-size: 11px;
    color: var(--bs-secondary-color, #6c757d);
  }

  .group-label {
    font-weight: 500;
    font-size: 11px;
  }
</style>
