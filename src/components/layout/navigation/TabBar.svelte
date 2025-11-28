<script>
  import { createEventDispatcher, onMount, afterUpdate } from "svelte";
  import TabContextMenu from "../../context-menus/TabContextMenu.svelte";
  import { invoke } from "@tauri-apps/api/core";

  const dispatch = createEventDispatcher();

  export let tabs = [];
  export let activeTab = null;

  let tabBarContainer;
  let visibleTabs = [];
  let overflowTabs = [];
  let showOverflowMenu = false;
  let overflowButton;
  let contextMenu = null; // { x, y, tab }

  function selectTab(tab) {
    dispatch("select", tab);
  }

  function closeTab(event, tab) {
    event.stopPropagation();
    dispatch("close", tab);
  }

  function getTabIcon(type) {
    switch (type) {
      case "query":
        return "fa-file-code";
      case "table":
        return "fa-table";
      case "connection":
        return "fa-database";
      default:
        return "fa-file";
    }
  }

  // Measure text width dynamically
  let canvas;
  let ctx;

  function getTextWidth(text) {
    if (!canvas) {
      canvas = document.createElement("canvas");
      ctx = canvas.getContext("2d");
    }
    // Use normal font weight for all tabs
    ctx.font = '12px system-ui, -apple-system, "Segoe UI", Roboto, sans-serif';
    return ctx.measureText(text).width;
  }

  // Calculate dynamic width based on actual text measurement
  function getTabWidth(title) {
    // Measure actual text width
    const textWidth = getTextWidth(title);

    // Components:
    // - icon: 11px
    // - gap after icon: 8px (gap-2)
    // - text: textWidth (measured with correct font weight)
    // - gap after text (if modified): 8px
    // - close button: 18px
    // - padding-left: 12px (ps-3)
    // - padding-right (margin-right on close): 8px
    // Total fixed space: 11 + 0 + 18 + 12 + 0 + some buffer = ~60px
    const fixedSpace = 65;
    const calculatedWidth = fixedSpace + textWidth;
    return calculatedWidth;
  }

  function calculateVisibleTabs() {
    if (!tabBarContainer || tabs.length === 0) {
      visibleTabs = tabs;
      overflowTabs = [];
      return;
    }

    const containerWidth = tabBarContainer.offsetWidth;
    // Reserve space for new tab button (32px) and overflow button (32px)
    const availableWidth = containerWidth - 64;

    let currentWidth = 0;
    let visible = [];
    let overflow = [];

    for (let i = 0; i < tabs.length; i++) {
      const tab = tabs[i];
      const isActive = activeTab?.id === tab.id;
      const tabWidth = getTabWidth(tab.title);

      if (currentWidth + tabWidth <= availableWidth || i === 0) {
        // Always show at least the first tab (active or first in list)
        visible.push(tab);
        currentWidth += tabWidth;
      } else {
        overflow.push(tab);
      }
    }

    // Ensure active tab is always visible
    if (activeTab && !visible.find((t) => t.id === activeTab.id)) {
      const activeIndex = tabs.findIndex((t) => t.id === activeTab.id);
      if (activeIndex !== -1) {
        // Remove last visible tab and add active tab
        if (visible.length > 0) {
          overflow.unshift(visible.pop());
        }
        visible.push(activeTab);
        overflow = overflow.filter((t) => t.id !== activeTab.id);
      }
    }

    visibleTabs = visible;
    overflowTabs = overflow;
  }

  function toggleOverflowMenu() {
    showOverflowMenu = !showOverflowMenu;
  }

  function handleClickOutside(event) {
    if (overflowButton && !overflowButton.contains(event.target)) {
      showOverflowMenu = false;
    }
  }

  function handleTabContextMenu(event, tab) {
    event.preventDefault();
    event.stopPropagation();

    contextMenu = {
      x: event.clientX,
      y: event.clientY,
      tab,
    };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  function handleContextMenuClose() {
    if (contextMenu) {
      dispatch("close", contextMenu.tab);
      closeContextMenu();
    }
  }

  function handleContextMenuCloseOthers() {
    if (contextMenu) {
      const tabsToClose = tabs.filter((t) => t.id !== contextMenu.tab.id);
      tabsToClose.forEach((tab) => dispatch("close", tab));
      closeContextMenu();
    }
  }

  function handleContextMenuCloseToLeft() {
    if (contextMenu) {
      const currentIndex = tabs.findIndex((t) => t.id === contextMenu.tab.id);
      const tabsToClose = tabs.slice(0, currentIndex);
      tabsToClose.forEach((tab) => dispatch("close", tab));
      closeContextMenu();
    }
  }

  function handleContextMenuCloseToRight() {
    if (contextMenu) {
      const currentIndex = tabs.findIndex((t) => t.id === contextMenu.tab.id);
      const tabsToClose = tabs.slice(currentIndex + 1);
      tabsToClose.forEach((tab) => dispatch("close", tab));
      closeContextMenu();
    }
  }

  function handleContextMenuCloseAll() {
    tabs.forEach((tab) => dispatch("close", tab));
    closeContextMenu();
  }

  function handleContextMenuDetach() {
    if (contextMenu) {
      // TODO: Implement detach functionality
      console.log("Detach tab:", contextMenu.tab);
      closeContextMenu();
    }
  }

  async function handleContextMenuCopyObjectName() {
    if (contextMenu) {
      try {
        // For table tabs, copy database.table format
        let textToCopy = contextMenu.tab.title;

        if (contextMenu.tab.type === "table" && contextMenu.tab.tableInfo) {
          const { database, name } = contextMenu.tab.tableInfo;
          textToCopy = `${database}.${name}`;
        }

        await invoke("copy_to_clipboard", { text: textToCopy });
      } catch (error) {
        console.error("Failed to copy:", error);
      }
      closeContextMenu();
    }
  }

  function handleContextMenuAddBookmark() {
    if (contextMenu) {
      // TODO: Implement bookmark functionality
      console.log("Add bookmark:", contextMenu.tab);
      closeContextMenu();
    }
  }

  function getContextMenuState() {
    if (!contextMenu) {
      return {
        canCloseLeft: false,
        canCloseRight: false,
        canCloseOthers: false,
      };
    }

    const currentIndex = tabs.findIndex((t) => t.id === contextMenu.tab.id);
    return {
      canCloseLeft: currentIndex > 0,
      canCloseRight: currentIndex < tabs.length - 1,
      canCloseOthers: tabs.length > 1,
    };
  }

  onMount(() => {
    calculateVisibleTabs();
    window.addEventListener("resize", calculateVisibleTabs);
    document.addEventListener("click", handleClickOutside);
    document.addEventListener("click", closeContextMenu);

    return () => {
      window.removeEventListener("resize", calculateVisibleTabs);
      document.removeEventListener("click", handleClickOutside);
      document.removeEventListener("click", closeContextMenu);
    };
  });

  afterUpdate(() => {
    calculateVisibleTabs();
  });

  $: if (tabs || activeTab) {
    calculateVisibleTabs();
  }
</script>

<div
  bind:this={tabBarContainer}
  class="tab-bar d-flex border-bottom"
  style="height: 32px; overflow: visible; user-select: none; flex-shrink: 0; position: relative; z-index: 100;"
>
  {#if tabs.length === 0}
    <!-- Empty state, no message -->
  {:else}
    <div
      class="d-flex flex-grow-1 align-items-stretch"
      style="overflow: hidden;"
    >
      {#each visibleTabs as tab (tab.id)}
        {@const isActive = activeTab?.id === tab.id}
        {@const tabWidth = getTabWidth(tab.title, isActive)}
        <button
          class="ps-3 tab-item d-flex align-items-center position-relative text-start {isActive
            ? 'tab-active'
            : ''}"
          style="width: {tabWidth}px; gap: 6px;"
          on:click={() => selectTab(tab)}
          on:contextmenu={(e) => handleTabContextMenu(e, tab)}
          title={tab.title}
        >
          <i
            class="fas {getTabIcon(tab.type)} tab-icon"
            style="font-size: 11px; flex-shrink: 0;"
          ></i>
          <span
            class="tab-title"
            style="font-size: 12px; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;"
          >
            {tab.title}
          </span>
          {#if tab.modified}
            <span
              class="modified-indicator"
              style="font-size: 16px; flex-shrink: 0;">●</span
            >
          {/if}
          <span
            class="tab-close-btn d-flex align-items-center justify-content-center"
            style="width: 18px; height: 18px; font-size: 9px; flex-shrink: 0;"
            on:click={(e) => closeTab(e, tab)}
            on:keydown={(e) => e.key === "Enter" && closeTab(e, tab)}
            title="Close"
            role="button"
            tabindex="0"
          >
            <i class="fas fa-times"></i>
          </span>
        </button>
      {/each}
    </div>

    <div class="d-flex align-items-center border-start">
      {#if overflowTabs.length > 0}
        <div class="position-relative" bind:this={overflowButton}>
          <button
            class="btn btn-sm p-0 border-0 bg-transparent overflow-btn"
            style="width: 32px; height: 32px; font-size: 12px;"
            title="More tabs ({overflowTabs.length})"
            on:click|stopPropagation={toggleOverflowMenu}
          >
            <i class="fas fa-chevron-down"></i>
          </button>

          {#if showOverflowMenu}
            <div
              class="overflow-menu position-absolute border shadow-sm"
              style="top: 100%; right: 0; min-width: 220px; max-width: 320px; max-height: 400px; overflow-y: auto; z-index: 99999;"
            >
              {#each overflowTabs as tab (tab.id)}
                <div
                  class="overflow-menu-item d-flex align-items-center gap-2 px-3 py-2 border-bottom {activeTab?.id ===
                  tab.id
                    ? 'bg-primary-subtle'
                    : ''}"
                  on:click={() => {
                    selectTab(tab);
                    showOverflowMenu = false;
                  }}
                  on:contextmenu={(e) => {
                    handleTabContextMenu(e, tab);
                    showOverflowMenu = false;
                  }}
                  on:keydown={(e) => e.key === "Enter" && selectTab(tab)}
                  role="button"
                  tabindex="0"
                >
                  <i
                    class="fas {getTabIcon(tab.type)}"
                    style="font-size: 11px; opacity: 0.6;"
                  ></i>
                  <span
                    class="flex-grow-1 text-truncate"
                    style="font-size: 12px;"
                  >
                    {tab.title}
                  </span>
                  {#if tab.modified}
                    <span class="text-primary" style="font-size: 16px;">●</span>
                  {/if}
                  <button
                    class="btn btn-sm p-0 border-0 bg-transparent text-secondary"
                    style="width: 18px; height: 18px; font-size: 10px; opacity: 0.6;"
                    on:click|stopPropagation={(e) => closeTab(e, tab)}
                    title="Close"
                  >
                    <i class="fas fa-times"></i>
                  </button>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>

{#if contextMenu}
  <TabContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    canCloseLeft={getContextMenuState().canCloseLeft}
    canCloseRight={getContextMenuState().canCloseRight}
    canCloseOthers={getContextMenuState().canCloseOthers}
    on:close={handleContextMenuClose}
    on:closeOthers={handleContextMenuCloseOthers}
    on:closeToLeft={handleContextMenuCloseToLeft}
    on:closeToRight={handleContextMenuCloseToRight}
    on:closeAll={handleContextMenuCloseAll}
    on:detach={handleContextMenuDetach}
    on:copyObjectName={handleContextMenuCopyObjectName}
    on:addBookmark={handleContextMenuAddBookmark}
  />
{/if}

<style>
  /* Tab Bar Container */
  .tab-bar {
    background: var(--tab-inactive-bg);
  }

  /* Tab Item */
  .tab-item {
    height: 32px;
    border: none;
    border-right: 1px solid var(--border-light);
    background: transparent;
    transition: all 0.15s ease;
    position: relative;
    cursor: pointer;
    outline: none;
  }

  .tab-item:hover {
    background: var(--tab-hover-bg);
  }

  .tab-item:focus {
    outline: none;
  }

  /* Active Tab */
  .tab-item.tab-active {
    background: var(--tab-active-bg);
    box-shadow: inset 0 2px 0 0 var(--accent-blue);
    margin-top: 0;
    z-index: 10;
  }

  .tab-item.tab-active::before {
    content: "";
    position: absolute;
    bottom: -1px;
    left: 0;
    right: 0;
    height: 1px;
    background: var(--tab-active-bg);
    z-index: 11;
  }

  /* Tab Icon */
  .tab-icon {
    color: var(--text-muted);
    opacity: 0.7;
    transition:
      opacity 0.15s,
      color 0.15s;
  }

  .tab-item:hover .tab-icon {
    color: var(--text-secondary);
    opacity: 0.85;
  }

  .tab-active .tab-icon {
    color: var(--accent-blue);
    opacity: 1;
  }

  /* Tab Title */
  .tab-title {
    color: var(--text-secondary);
    transition: color 0.15s;
  }

  .tab-active .tab-title {
    color: var(--text-primary);
  }

  /* Modified Indicator */
  .modified-indicator {
    color: var(--accent-blue);
    flex-shrink: 0;
    margin-left: -2px;
  }

  /* Close Button */
  .tab-close-btn {
    opacity: 0.35;
    transition: all 0.15s;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .tab-item:hover .tab-close-btn,
  .tab-item.tab-active .tab-close-btn {
    opacity: 0.5;
  }

  .tab-close-btn:hover {
    opacity: 1 !important;
    background-color: var(--hover-bg) !important;
    border-radius: 2px;
    transform: scale(1.05);
  }

  .tab-close-btn:active {
    background-color: var(--active-bg) !important;
  }

  /* Overflow Button */
  .overflow-btn {
    color: var(--text-secondary);
    transition: all 0.2s;
  }

  .overflow-btn:hover {
    background-color: var(--hover-bg) !important;
    color: var(--text-primary);
  }

  /* Overflow Menu */
  .overflow-menu {
    border-radius: 4px;
    margin-top: 4px;
    background: var(--bg-dropdown);
    border: 1px solid var(--border-color);
    box-shadow: var(--shadow-dropdown);
    color: var(--text-primary);
  }

  .overflow-menu-item {
    cursor: pointer;
    transition: background-color 0.15s;
    color: var(--text-primary);
  }

  .overflow-menu-item i.fas:not(.fa-times) {
    color: var(--text-muted);
  }

  .overflow-menu-item:hover {
    background-color: var(--hover-bg) !important;
  }

  .overflow-menu-item:hover i.fas:not(.fa-times) {
    color: var(--text-secondary);
  }

  .overflow-menu-item:last-child {
    border-bottom: none !important;
  }

  .overflow-menu-item button {
    opacity: 0;
    transition: opacity 0.2s;
    color: var(--text-secondary);
  }

  .overflow-menu-item:hover button {
    opacity: 0.6;
  }

  .overflow-menu-item button:hover {
    opacity: 1 !important;
    background-color: var(--hover-bg) !important;
    border-radius: 3px;
  }
</style>
