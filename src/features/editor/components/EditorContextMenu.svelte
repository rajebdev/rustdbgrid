<script>
  import { onDestroy } from "svelte";
  import { fly } from "svelte/transition";
  import { quintOut } from "svelte/easing";

  let visible = false;
  let x = 0;
  let y = 0;
  let menuItems = [];
  let menuElement;
  let adjustedX = 0;
  let adjustedY = 0;

  export function show(event, items) {
    event.preventDefault();
    x = event.clientX;
    y = event.clientY;
    menuItems = items;
    visible = true;

    // Adjust position after render
    setTimeout(() => {
      adjustPosition();
    }, 0);
  }

  export function hide() {
    visible = false;
  }

  function adjustPosition() {
    if (!menuElement) return;

    const rect = menuElement.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    // Center vertically relative to click position
    adjustedY = y - rect.height / 2;

    // Ensure menu stays within viewport
    const topOffset = 60;
    const bottomOffset = 30;
    if (adjustedY < topOffset) {
      adjustedY = topOffset;
    } else if (adjustedY + rect.height > viewportHeight - bottomOffset) {
      adjustedY = viewportHeight - rect.height - bottomOffset;
    }

    // Adjust horizontally if needed
    adjustedX = x;
    if (x + rect.width > viewportWidth - 10) {
      adjustedX = x - rect.width;
    }
    if (adjustedX < 10) {
      adjustedX = 10;
    }
  }

  function handleClick(item) {
    if (item.onClick) {
      item.onClick();
    }
    hide();
  }

  function handleKeydown(e) {
    if (e.key === "Escape") {
      hide();
    }
  }

  function handleWindowClick() {
    hide();
  }

  onDestroy(() => {
    document.removeEventListener("click", handleWindowClick);
    document.removeEventListener("keydown", handleKeydown);
  });
</script>

<svelte:window on:click={handleWindowClick} on:keydown={handleKeydown} />

{#if visible}
  <div
    bind:this={menuElement}
    class="context-menu"
    style="position: fixed; left: {adjustedX || x}px; top: {adjustedY ||
      y}px; z-index: 10000;"
    role="menu"
    tabindex="-1"
    on:click|stopPropagation
    on:keydown|stopPropagation={(e) => {
      if (e.key === "Escape") hide();
    }}
    in:fly={{ y: -10, duration: 200, easing: quintOut }}
  >
    {#each menuItems as item (item.id)}
      {#if item.label === "---"}
        <div class="context-menu-divider"></div>
      {:else}
        <button
          class="context-menu-item"
          class:disabled={item.disabled}
          on:click={() => !item.disabled && handleClick(item)}
          role="menuitem"
          disabled={item.disabled}
          title={item.title || ""}
        >
          {#if item.icon}
            <i class={item.icon}></i>
          {/if}
          <span>{item.label}</span>
          {#if item.shortcut}
            <kbd>{item.shortcut}</kbd>
          {/if}
        </button>
      {/if}
    {/each}
  </div>
{/if}

<style>
  .context-menu {
    background: var(--bg-dropdown);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    box-shadow: var(--shadow-dropdown);
    min-width: 240px;
    padding: 4px;
    font-size: 12px;
  }

  .context-menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: transparent;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
    border-radius: 4px;
    transition: background-color 0.15s;
    width: 100%;
    white-space: nowrap;
  }

  .context-menu-item:hover:not(:disabled) {
    background: var(--hover-bg);
  }

  .context-menu-item:disabled,
  .context-menu-item.disabled {
    color: var(--text-muted);
    cursor: not-allowed;
    opacity: 0.6;
  }

  .context-menu-item i {
    width: 16px;
    font-size: 11px;
    text-align: center;
  }

  .context-menu-item span {
    flex: 1;
  }

  .context-menu-item kbd {
    font-size: 10px;
    padding: 2px 6px;
    background: var(--bg-tertiary);
    border-radius: 3px;
    color: var(--text-secondary);
    font-family: monospace;
    margin-left: auto;
  }

  .context-menu-divider {
    height: 1px;
    background: var(--border-color);
    margin: 4px 0;
  }
</style>
