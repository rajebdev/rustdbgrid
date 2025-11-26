<script>
  import { createEventDispatcher, onMount } from "svelte";
  import { fly } from "svelte/transition";
  import { quintOut } from "svelte/easing";

  export let x = 0;
  export let y = 0;
  export let database = null;
  export let connection = null;

  const dispatch = createEventDispatcher();

  let menuElement;
  let adjustedX = x;
  let adjustedY = y;

  onMount(() => {
    // Center menu vertically relative to click position
    if (menuElement) {
      const rect = menuElement.getBoundingClientRect();
      const viewportWidth = window.innerWidth;
      const viewportHeight = window.innerHeight;

      // Center vertically (same distance top and bottom)
      adjustedY = y - rect.height / 2;

      // Ensure menu stays within viewport vertically
      // Reserve space at top for toolbar/header (60px) and bottom for status bar (30px)
      const topOffset = 60;
      const bottomOffset = 30;
      if (adjustedY < topOffset) {
        adjustedY = topOffset;
      } else if (adjustedY + rect.height > viewportHeight - bottomOffset) {
        adjustedY = viewportHeight - rect.height - bottomOffset;
      }

      // Adjust horizontally if needed
      adjustedX = x;
      if (rect.right > viewportWidth) {
        adjustedX = x - rect.width;
      }
      if (adjustedX < 10) {
        adjustedX = 10;
      }
    }
  });

  function handleSqlEditor() {
    dispatch("sqlEditor", { database, connection });
  }

  function handleCreate() {
    dispatch("create", { database, connection });
  }

  function handleViewDatabase() {
    dispatch("viewDatabase", { database, connection });
  }

  function handleFilter() {
    dispatch("filter", { database, connection });
  }

  function handleCompareMigrate() {
    dispatch("compareMigrate", { database, connection });
  }

  function handleTools() {
    dispatch("tools", { database, connection });
  }

  function handleCopy() {
    dispatch("copy", { database, connection });
  }

  function handlePaste() {
    dispatch("paste", { database, connection });
  }

  function handleCopyAdvancedInfo() {
    dispatch("copyAdvancedInfo", { database, connection });
  }

  function handleDelete() {
    dispatch("delete", { database, connection });
  }

  function handleRename() {
    dispatch("rename", { database, connection });
  }

  function handleRefresh() {
    dispatch("refresh", { database, connection });
  }
</script>

<div
  bind:this={menuElement}
  class="context-menu"
  style="left: {adjustedX}px; top: {adjustedY}px;"
  role="menu"
  tabindex="-1"
  on:click|stopPropagation
  on:keydown|stopPropagation
  in:fly={{ y: -10, duration: 200, easing: quintOut }}
>
  <button class="menu-item" on:click={handleSqlEditor}>
    <i class="fas fa-terminal"></i>
    <span>SQL Editor</span>
    <span class="shortcut">F4</span>
  </button>

  <button class="menu-item has-submenu">
    <i class="fas fa-plus"></i>
    <span>Create</span>
    <i class="fas fa-chevron-right submenu-arrow"></i>
  </button>

  <button class="menu-item" on:click={handleViewDatabase}>
    <i class="fas fa-eye"></i>
    <span>View Database</span>
    <span class="shortcut">F4</span>
  </button>

  <button class="menu-item has-submenu">
    <i class="fas fa-filter"></i>
    <span>Filter</span>
    <i class="fas fa-chevron-right submenu-arrow"></i>
  </button>

  <button class="menu-item has-submenu">
    <i class="fas fa-code-compare"></i>
    <span>Compare/Migrate</span>
    <i class="fas fa-chevron-right submenu-arrow"></i>
  </button>

  <button class="menu-item has-submenu">
    <i class="fas fa-wrench"></i>
    <span>Tools</span>
    <i class="fas fa-chevron-right submenu-arrow"></i>
  </button>

  <div class="menu-separator"></div>

  <button class="menu-item" on:click={handleCopy}>
    <i class="fas fa-copy"></i>
    <span>Copy</span>
    <span class="shortcut">Ctrl+C</span>
  </button>

  <button class="menu-item" on:click={handlePaste}>
    <i class="fas fa-paste"></i>
    <span>Paste</span>
    <span class="shortcut">Ctrl+V</span>
  </button>

  <button class="menu-item" on:click={handleCopyAdvancedInfo}>
    <i class="fas fa-info-circle"></i>
    <span>Copy Advanced Info</span>
    <span class="shortcut">Ctrl+Shift+C</span>
  </button>

  <div class="menu-separator"></div>

  <button class="menu-item delete-item" on:click={handleDelete}>
    <i class="fas fa-trash"></i>
    <span>Delete</span>
    <span class="shortcut">Delete</span>
  </button>

  <button class="menu-item" on:click={handleRename}>
    <i class="fas fa-edit"></i>
    <span>Rename</span>
    <span class="shortcut">F2</span>
  </button>

  <div class="menu-separator"></div>

  <button class="menu-item" on:click={handleRefresh}>
    <i class="fas fa-sync"></i>
    <span>Refresh</span>
    <span class="shortcut">F5</span>
  </button>
</div>

<style>
  .context-menu {
    position: fixed;
    background: white;
    border: 1px solid #dee2e6;
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    z-index: 10000;
    min-width: 240px;
    padding: 4px;
    font-size: 12px;
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    cursor: pointer;
    transition: background-color 0.15s;
    width: 100%;
    border: none;
    background: transparent;
    color: #212529;
    text-align: left;
    font-family: inherit;
    font-size: 12px;
    border-radius: 4px;
    white-space: nowrap;
  }

  .menu-item:hover {
    background: #f8f9fa;
  }

  .menu-item i:first-child {
    width: 16px;
    font-size: 11px;
    text-align: center;
  }

  .menu-item span:not(.shortcut):not(.submenu-arrow) {
    flex: 1;
  }

  .menu-item .shortcut {
    font-size: 10px;
    padding: 2px 6px;
    background: #e9ecef;
    border-radius: 3px;
    color: #6c757d;
    font-family: monospace;
    margin-left: auto;
  }

  .menu-item.has-submenu {
    padding-right: 8px;
  }

  .submenu-arrow {
    font-size: 10px;
    color: #adb5bd;
    margin-left: auto;
  }

  .menu-item.delete-item {
    color: #dc3545;
  }

  .menu-item.delete-item:hover {
    background: #fff5f5;
  }

  .menu-separator {
    height: 1px;
    background: #dee2e6;
    margin: 4px 0;
  }
</style>
