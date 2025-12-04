<script>
  import { createEventDispatcher, onMount } from "svelte";
  import { fly } from "svelte/transition";
  import { quintOut } from "svelte/easing";
  import ContextMenuItem from "./ContextMenuItem.svelte";

  export let x = 0;
  export let y = 0;
  export let items = [];
  export let visible = true;

  const dispatch = createEventDispatcher();

  let menuElement;
  let adjustedX = x;
  let adjustedY = y;

  onMount(() => {
    adjustPosition();
  });

  function adjustPosition() {
    if (!menuElement) return;

    const rect = menuElement.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    // Center vertically relative to click position
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

  function handleItemClick(item) {
    if (item.disabled) return;
    dispatch("action", { action: item.action, item });
  }

  function handleClose() {
    dispatch("close");
  }
</script>

{#if visible}
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
    {#each items as item, index (item.id || `${item.type}-${index}`)}
      <ContextMenuItem {item} onClick={() => handleItemClick(item)} />
    {/each}
  </div>
{/if}

<style>
  .context-menu {
    position: fixed;
    background: var(--bg-dropdown);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    box-shadow: var(--shadow-dropdown);
    z-index: 10000;
    min-width: 240px;
    padding: 4px;
    font-size: 12px;
  }
</style>
