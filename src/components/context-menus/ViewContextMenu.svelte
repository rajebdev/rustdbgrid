<script>
  import { createEventDispatcher, onMount } from "svelte";
  import { fly } from "svelte/transition";
  import { quintOut } from "svelte/easing";

  export let x = 0;
  export let y = 0;
  export let view = null;
  export let connection = null;
  export let database = null;

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

  function handleAction(type) {
    dispatch(type, { view, connection, database });
  }
</script>

<div
  bind:this={menuElement}
  class="context-menu dropdown-menu show"
  style="position: fixed; left: {adjustedX}px; top: {adjustedY}px; z-index: 9999;"
  role="menu"
  tabindex="-1"
  on:click|stopPropagation
  on:keydown|stopPropagation
  in:fly={{ y: -10, duration: 200, easing: quintOut }}
>
  <!-- View Structure -->
  <button
    class="dropdown-item d-flex align-items-center justify-content-between"
    on:click={() => handleAction("viewStructure")}
  >
    <span>
      <i class="fas fa-eye me-2"></i>
      View Structure
    </span>
    <span class="text-muted ms-3 small">F4</span>
  </button>

  <!-- View Definition -->
  <button class="dropdown-item" on:click={() => handleAction("viewDefinition")}>
    <i class="fas fa-code me-2"></i>
    View Definition
  </button>

  <!-- View Data -->
  <button class="dropdown-item" on:click={() => handleAction("viewData")}>
    <i class="fas fa-table me-2"></i>
    View Data
  </button>

  <div class="dropdown-divider"></div>

  <!-- Export Data -->
  <button class="dropdown-item" on:click={() => handleAction("exportData")}>
    <i class="fas fa-file-export me-2"></i>
    Export Data
  </button>

  <!-- Import Data -->
  <button class="dropdown-item" on:click={() => handleAction("importData")}>
    <i class="fas fa-file-import me-2"></i>
    Import Data
  </button>

  <div class="dropdown-divider"></div>

  <!-- Read in Console -->
  <button class="dropdown-item" on:click={() => handleAction("readInConsole")}>
    <i class="fas fa-terminal me-2"></i>
    Read in Console
  </button>

  <div class="dropdown-divider"></div>

  <!-- Copy -->
  <button class="dropdown-item" on:click={() => handleAction("copy")}>
    <i class="fas fa-copy me-2"></i>
    Copy
  </button>

  <!-- Copy Advanced Info -->
  <button
    class="dropdown-item"
    on:click={() => handleAction("copyAdvancedInfo")}
  >
    <i class="fas fa-info-circle me-2"></i>
    Copy Advanced Info
  </button>

  <div class="dropdown-divider"></div>

  <!-- Rename -->
  <button class="dropdown-item" on:click={() => handleAction("rename")}>
    <i class="fas fa-edit me-2"></i>
    Rename
  </button>

  <!-- Delete -->
  <button
    class="dropdown-item text-danger"
    on:click={() => handleAction("delete")}
  >
    <i class="fas fa-trash me-2"></i>
    Delete
  </button>

  <div class="dropdown-divider"></div>

  <!-- Refresh -->
  <button class="dropdown-item" on:click={() => handleAction("refresh")}>
    <i class="fas fa-sync-alt me-2"></i>
    Refresh
  </button>
</div>

<style>
  .context-menu {
    font-size: 12px;
    min-width: 220px;
    max-width: 300px;
    padding: 4px 0;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .dropdown-item {
    display: block;
    width: 100%;
    padding: 6px 16px;
    clear: both;
    font-weight: 400;
    color: var(--text-primary);
    text-align: left;
    white-space: nowrap;
    background-color: transparent;
    border: 0;
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .dropdown-item:hover:not(:disabled) {
    background-color: var(--hover-bg);
  }

  .dropdown-item:disabled {
    color: var(--text-muted);
    cursor: not-allowed;
    opacity: 0.5;
  }

  .dropdown-item.text-danger:hover {
    background-color: var(--danger-bg);
    color: var(--danger-text);
  }

  .dropdown-divider {
    height: 0;
    margin: 4px 0;
    overflow: hidden;
    border-top: 1px solid var(--border-color);
  }

  .small {
    font-size: 10px;
  }

  i {
    width: 16px;
    text-align: center;
  }
</style>
