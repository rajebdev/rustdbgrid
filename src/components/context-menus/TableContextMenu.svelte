<script>
  import { createEventDispatcher, onMount } from "svelte";
  import { fly } from "svelte/transition";
  import { quintOut } from "svelte/easing";

  export let x = 0;
  export let y = 0;
  export let table = null;
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
    dispatch(type, { table, connection, database });
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
  <!-- View Table -->
  <button
    class="dropdown-item d-flex align-items-center justify-content-between"
    on:click={() => handleAction("viewTable")}
  >
    <span>
      <i class="fas fa-table me-2"></i>
      View Table
    </span>
    <span class="text-muted ms-3 small">F4</span>
  </button>

  <!-- Filter -->
  <li class="dropdown-submenu">
    <button
      class="dropdown-item d-flex align-items-center justify-content-between"
      disabled
    >
      <span>
        <i class="fas fa-filter me-2"></i>
        Filter (TODO)
      </span>
      <i class="fas fa-chevron-right ms-auto"></i>
    </button>
  </li>

  <!-- View Diagram -->
  <button
    class="dropdown-item d-flex align-items-center justify-content-between"
    on:click={() => handleAction("viewDiagram")}
  >
    <span>
      <i class="fas fa-project-diagram me-2"></i>
      View Diagram
    </span>
    <span class="text-muted ms-3 small">Ctrl+Shift+Enter</span>
  </button>

  <!-- View Data -->
  <button class="dropdown-item" on:click={() => handleAction("viewData")}>
    <i class="fas fa-eye me-2"></i>
    View Data
  </button>

  <div class="dropdown-divider"></div>

  <!-- Compare/Migrate -->
  <li class="dropdown-submenu">
    <button
      class="dropdown-item d-flex align-items-center justify-content-between"
      disabled
    >
      <span>
        <i class="fas fa-exchange-alt me-2"></i>
        Compare/Migrate (TODO)
      </span>
      <i class="fas fa-chevron-right ms-auto"></i>
    </button>
  </li>

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

  <!-- Tools -->
  <li class="dropdown-submenu">
    <button
      class="dropdown-item d-flex align-items-center justify-content-between"
      disabled
    >
      <span>
        <i class="fas fa-tools me-2"></i>
        Tools (TODO)
      </span>
      <i class="fas fa-chevron-right ms-auto"></i>
    </button>
  </li>

  <!-- Generate SQL -->
  <li class="dropdown-submenu">
    <button
      class="dropdown-item d-flex align-items-center justify-content-between"
      disabled
    >
      <span>
        <i class="fas fa-code me-2"></i>
        Generate SQL (TODO)
      </span>
      <i class="fas fa-chevron-right ms-auto"></i>
    </button>
  </li>

  <!-- Read data in SQL console -->
  <button class="dropdown-item" on:click={() => handleAction("readInConsole")}>
    <i class="fas fa-terminal me-2"></i>
    Read data in SQL console
  </button>

  <div class="dropdown-divider"></div>

  <!-- Copy -->
  <button
    class="dropdown-item d-flex align-items-center justify-content-between"
    on:click={() => handleAction("copy")}
  >
    <span>
      <i class="fas fa-copy me-2"></i>
      Copy
    </span>
    <span class="text-muted ms-3 small">Ctrl+C</span>
  </button>

  <!-- Paste -->
  <button
    class="dropdown-item d-flex align-items-center justify-content-between"
    on:click={() => handleAction("paste")}
  >
    <span>
      <i class="fas fa-paste me-2"></i>
      Paste
    </span>
    <span class="text-muted ms-3 small">Ctrl+V</span>
  </button>

  <!-- Copy Advanced Info -->
  <button
    class="dropdown-item d-flex align-items-center justify-content-between"
    on:click={() => handleAction("copyAdvancedInfo")}
  >
    <span>
      <i class="fas fa-info-circle me-2"></i>
      Copy Advanced Info
    </span>
    <span class="text-muted ms-3 small">Ctrl+Shift+C</span>
  </button>

  <div class="dropdown-divider"></div>

  <!-- Delete -->
  <button
    class="dropdown-item d-flex align-items-center justify-content-between text-danger"
    on:click={() => handleAction("delete")}
  >
    <span>
      <i class="fas fa-trash me-2"></i>
      Delete
    </span>
    <span class="text-muted ms-3 small">Delete</span>
  </button>

  <!-- Rename -->
  <button
    class="dropdown-item d-flex align-items-center justify-content-between"
    on:click={() => handleAction("rename")}
  >
    <span>
      <i class="fas fa-edit me-2"></i>
      Rename
    </span>
    <span class="text-muted ms-3 small">F2</span>
  </button>

  <div class="dropdown-divider"></div>

  <!-- Refresh -->
  <button
    class="dropdown-item d-flex align-items-center justify-content-between"
    on:click={() => handleAction("refresh")}
  >
    <span>
      <i class="fas fa-sync-alt me-2"></i>
      Refresh
    </span>
    <span class="text-muted ms-3 small">F5</span>
  </button>
</div>

<style>
  .context-menu {
    min-width: 250px;
    box-shadow: var(--shadow-dropdown);
    border: 1px solid var(--border-color);
    padding: 4px 0;
    background: var(--bg-dropdown);
  }

  .dropdown-item {
    padding: 6px 16px;
    font-size: 13px;
    cursor: pointer;
    border: none;
    background: none;
    width: 100%;
    text-align: left;
    display: flex;
    align-items: center;
    color: var(--text-primary);
  }

  .dropdown-item:hover {
    background-color: var(--hover-bg);
  }

  .dropdown-item:active {
    background-color: var(--active-bg);
  }

  .dropdown-item.text-danger:hover {
    background-color: var(--accent-red-light);
    color: var(--accent-red);
  }

  .dropdown-item i {
    width: 16px;
    text-align: center;
  }

  .dropdown-divider {
    height: 1px;
    margin: 4px 0;
    background-color: var(--border-light);
    border: none;
  }

  .dropdown-submenu {
    list-style: none;
    position: relative;
  }

  .dropdown-submenu button {
    width: 100%;
  }

  .small {
    font-size: 11px;
  }
</style>
