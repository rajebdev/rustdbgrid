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
    class="context-menu dropdown-menu show"
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
        <div class="dropdown-divider"></div>
      {:else}
        <button
          class="dropdown-item d-flex align-items-center justify-content-between"
          class:disabled={item.disabled}
          on:click={() => !item.disabled && handleClick(item)}
          role="menuitem"
          disabled={item.disabled}
          title={item.title || ""}
        >
          <span class="d-flex align-items-center">
            {#if item.icon}
              <i class="{item.icon} me-2"></i>
            {/if}
            <span>{item.label}</span>
          </span>
          {#if item.shortcut}
            <span class="text-muted ms-3 small">{item.shortcut}</span>
          {/if}
        </button>
      {/if}
    {/each}
  </div>
{/if}

<style>
  .context-menu {
    min-width: 250px;
    box-shadow: var(--shadow-dropdown);
    border: 1px solid var(--border-color);
    padding: 4px 0;
    background: var(--bg-dropdown);
    border-radius: 6px;
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
    transition: background-color 0.15s ease;
  }

  .dropdown-item:hover:not(:disabled) {
    background-color: var(--hover-bg);
  }

  .dropdown-item:focus:not(:disabled) {
    background-color: var(--accent-blue-light);
    outline: none;
  }

  .dropdown-item:disabled,
  .dropdown-item.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .dropdown-item i {
    width: 16px;
    text-align: center;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .dropdown-item:hover:not(:disabled) i {
    color: var(--text-primary);
  }

  .dropdown-divider {
    height: 1px;
    margin: 4px 0;
    background-color: var(--border-light);
    border: none;
  }

  .small {
    font-size: 11px;
  }
</style>
