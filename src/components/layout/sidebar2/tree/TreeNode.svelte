<script>
  import { createEventDispatcher } from "svelte";

  export let expanded = false;
  export let loading = false;
  export let active = false;
  export let selectable = true;
  export let indent = 0;

  const dispatch = createEventDispatcher();

  function handleToggle(e) {
    e.stopPropagation();
    dispatch("toggle");
  }

  function handleClick(e) {
    if (selectable) {
      dispatch("click", e);
    }
  }

  function handleContextMenu(e) {
    dispatch("contextmenu", e);
  }
</script>

<div class="tree-node" style="--indent: {indent}">
  <!-- svelte-ignore a11y-role-has-required-aria-props -->
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <div
    class="tree-node-content"
    class:active
    class:expanded
    on:click={handleClick}
    on:contextmenu|preventDefault={handleContextMenu}
    on:dblclick={(e) => dispatch("dblclick", e)}
    role="treeitem"
    tabindex="0"
    aria-expanded={expanded}
  >
    <!-- Toggle Button -->
    <button
      class="tree-toggle"
      on:click={handleToggle}
      aria-label="Toggle"
      disabled={loading}
    >
      {#if loading}
        <i class="fas fa-spinner fa-spin"></i>
      {:else if $$slots.children || $$slots.default}
        <i class="fas fa-chevron-{expanded ? 'down' : 'right'}"></i>
      {:else}
        <span class="toggle-placeholder"></span>
      {/if}
    </button>

    <!-- Icon Slot -->
    <span class="tree-icon">
      <slot name="icon">
        <i class="fas fa-folder"></i>
      </slot>
    </span>

    <!-- Label Slot -->
    <span class="tree-label">
      <slot name="label">Label</slot>
    </span>

    <!-- Badge/Extra Slot -->
    {#if $$slots.badge}
      <span class="tree-badge">
        <slot name="badge"></slot>
      </span>
    {/if}

    <!-- Actions Slot (shown on hover) -->
    {#if $$slots.actions}
      <span class="tree-actions">
        <slot name="actions"></slot>
      </span>
    {/if}
  </div>

  <!-- Children Slot -->
  {#if expanded && $$slots.children}
    <div class="tree-children">
      <slot name="children"></slot>
    </div>
  {/if}
</div>

<style>
  .tree-node {
    --node-indent: calc(var(--indent, 0) * 12px);
  }

  .tree-node-content {
    display: flex;
    align-items: center;
    padding: 2px 4px 2px calc(4px + var(--node-indent));
    cursor: pointer;
    border-radius: 4px;
    gap: 4px;
    min-height: 24px;
    transition: background-color 0.15s ease;
  }

  .tree-node-content:hover {
    background-color: var(--bs-tertiary-bg, rgba(0, 0, 0, 0.05));
  }

  .tree-node-content.active {
    background-color: var(--bs-primary-bg-subtle, rgba(13, 110, 253, 0.1));
  }

  .tree-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    padding: 0;
    border: none;
    background: transparent;
    color: var(--bs-secondary-color, #6c757d);
    font-size: 10px;
    cursor: pointer;
    flex-shrink: 0;
  }

  .tree-toggle:hover:not(:disabled) {
    color: var(--bs-body-color, #212529);
  }

  .tree-toggle:disabled {
    cursor: default;
  }

  .toggle-placeholder {
    width: 10px;
  }

  .tree-icon {
    display: flex;
    align-items: center;
    font-size: 12px;
    flex-shrink: 0;
  }

  .tree-label {
    flex: 1;
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .tree-badge {
    font-size: 10px;
    color: var(--bs-secondary-color, #6c757d);
    flex-shrink: 0;
  }

  .tree-actions {
    display: none;
    gap: 2px;
    flex-shrink: 0;
  }

  .tree-node-content:hover .tree-actions {
    display: flex;
  }
</style>
