<script>
  import { createEventDispatcher } from "svelte";
  import Sidebar from "../navigation/Sidebar.svelte";

  const dispatch = createEventDispatcher();

  export let show = true;
  export let width = 320;
  export let minWidth = 200;
  export let maxWidth = 600;
  export let isResizing = false;

  // Suppress unused export warnings - these are used by parent for validation
  $: void minWidth;
  $: void maxWidth;

  function handleMouseDown(event) {
    isResizing = true;
    dispatch("startResize", { event });
    event.preventDefault();
  }

  function toggleSidebar() {
    show = !show;
    dispatch("toggle");
  }

  function forwardEvent(event) {
    dispatch(event.type, event.detail);
  }
</script>

{#if show}
  <div
    class="sidebar-container border-end"
    style="width: {width}px; flex-shrink: 0; position: relative;"
  >
    <Sidebar on:openTableTab={forwardEvent} />
    <button
      class="resize-handle"
      on:mousedown={handleMouseDown}
      aria-label="Resize sidebar"
      class:resizing={isResizing}
    ></button>
  </div>
{:else}
  <button
    class="sidebar-toggle-button btn btn-sm btn-primary"
    on:click={toggleSidebar}
    title="Show Sidebar (Ctrl+B)"
  >
    <i class="fas fa-chevron-right"></i>
  </button>
{/if}

<style>
  .sidebar-container {
    position: relative;
  }

  .resize-handle {
    position: absolute;
    top: 0;
    right: 0;
    width: 4px;
    height: 100%;
    cursor: col-resize;
    background-color: transparent;
    transition: background-color 0.2s;
    z-index: 10;
    border: none;
    padding: 0;
  }

  .resize-handle:hover,
  .resize-handle.resizing {
    background-color: var(--accent-blue);
  }

  .resize-handle::before {
    content: "";
    position: absolute;
    top: 0;
    left: -2px;
    width: 8px;
    height: 100%;
  }

  .sidebar-toggle-button {
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    z-index: 1000;
    border-radius: 0 4px 4px 0;
    padding: 8px 6px;
    font-size: 12px;
    box-shadow: var(--shadow-md);
    border-left: none;
  }

  .sidebar-toggle-button:hover {
    padding-left: 8px;
    transition: padding 0.2s ease;
  }
</style>
