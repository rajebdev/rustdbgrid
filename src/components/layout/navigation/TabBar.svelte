<script>
  import { createEventDispatcher, onMount, afterUpdate } from "svelte";

  const dispatch = createEventDispatcher();

  export let tabs = [];
  export let activeTab = null;

  let tabBarContainer;
  let visibleTabs = [];
  let overflowTabs = [];
  let showOverflowMenu = false;
  let overflowButton;

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
      ctx.font =
        '12px system-ui, -apple-system, "Segoe UI", Roboto, sans-serif';
    }
    return ctx.measureText(text).width;
  }

  // Calculate dynamic width based on actual text measurement
  function getTabWidth(title) {
    // Measure actual text width
    const textWidth = getTextWidth(title);

    // Components: icon (12px) + gaps (0.5rem * 3 = 24px) + close button (18px) + padding-left (1rem) + padding-right (1rem) = 16px
    // Total fixed space: 12 + 24 + 18 + 16 = 70px
    const fixedSpace = 70;
    const calculatedWidth = fixedSpace + textWidth;

    // Min 80px, max 250px
    return Math.min(Math.max(calculatedWidth, 80), 250);
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

  onMount(() => {
    calculateVisibleTabs();
    window.addEventListener("resize", calculateVisibleTabs);
    document.addEventListener("click", handleClickOutside);

    return () => {
      window.removeEventListener("resize", calculateVisibleTabs);
      document.removeEventListener("click", handleClickOutside);
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
  class="d-flex bg-body-secondary border-bottom"
  style="height: 32px; overflow: visible; user-select: none; flex-shrink: 0; position: relative; z-index: 100;"
>
  {#if tabs.length === 0}
    <!-- Empty state, no message -->
  {:else}
    <div class="d-flex flex-grow-1 align-items-end" style="overflow: hidden;">
      {#each visibleTabs as tab (tab.id)}
        <div
          class="d-flex align-items-center gap-2 px-3 border-end position-relative {activeTab?.id ===
          tab.id
            ? 'bg-body border-top border-2 border-primary'
            : ''}"
          style="width: {getTabWidth(tab.title)}px; height: {activeTab?.id ===
          tab.id
            ? '32px'
            : '28px'}; transition: all 0.15s; overflow: hidden; {activeTab?.id ===
          tab.id
            ? 'font-weight: 500;'
            : ''}"
        >
          <button
            class="d-flex align-items-center gap-2 border-0 bg-transparent p-0 text-start"
            style="cursor: pointer; min-width: 0; flex-shrink: 1;"
            on:click={() => selectTab(tab)}
            title={tab.title}
          >
            <i
              class="fas {getTabIcon(tab.type)} text-secondary"
              style="font-size: 12px; flex-shrink: 0;"
            ></i>
            <span
              class="text-truncate"
              style="font-size: 12px; flex-shrink: 1; min-width: 0;"
            >
              {tab.title}
            </span>
            {#if tab.modified}
              <span
                class="text-primary"
                style="font-size: 14px; flex-shrink: 0;">●</span
              >
            {/if}
          </button>
          <button
            class="btn btn-sm p-0 border-0 bg-transparent text-secondary"
            style="width: 18px; height: 18px; font-size: 11px; flex-shrink: 0;"
            on:click={(e) => closeTab(e, tab)}
            title="Close"
          >
            <i class="fas fa-times"></i>
          </button>
        </div>
      {/each}
    </div>

    <div class="d-flex align-items-center border-start">
      {#if overflowTabs.length > 0}
        <div class="position-relative" bind:this={overflowButton}>
          <button
            class="btn btn-sm p-1 border-0 bg-transparent text-secondary"
            style="width: 32px; height: 24px; font-size: 12px;"
            title="More tabs ({overflowTabs.length})"
            on:click|stopPropagation={toggleOverflowMenu}
          >
            <i class="fas fa-chevron-down"></i>
          </button>

          {#if showOverflowMenu}
            <div
              class="position-absolute bg-body border shadow-sm"
              style="top: 100%; right: 0; min-width: 200px; max-width: 300px; max-height: 400px; overflow-y: auto; z-index: 99999;"
            >
              {#each overflowTabs as tab (tab.id)}
                <div
                  class="d-flex align-items-center gap-2 px-3 py-2 border-bottom overflow-tab-item {activeTab?.id ===
                  tab.id
                    ? 'bg-primary-subtle'
                    : ''}"
                  style="cursor: pointer;"
                  on:click={() => {
                    selectTab(tab);
                    showOverflowMenu = false;
                  }}
                  on:keydown={(e) => e.key === "Enter" && selectTab(tab)}
                  role="button"
                  tabindex="0"
                >
                  <i
                    class="fas {getTabIcon(tab.type)} text-secondary"
                    style="font-size: 12px;"
                  ></i>
                  <span
                    class="flex-grow-1 text-truncate"
                    style="font-size: 12px;"
                  >
                    {tab.title}
                  </span>
                  {#if tab.modified}
                    <span class="text-primary" style="font-size: 14px;">●</span>
                  {/if}
                  <button
                    class="btn btn-sm p-0 border-0 bg-transparent text-secondary"
                    style="width: 18px; height: 18px; font-size: 11px;"
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

<style>
  /* Hover effect for overflow menu items */
  .overflow-tab-item:hover {
    background-color: #e9ecef !important;
  }

  /* Hover effect for close button */
  .btn:hover {
    background-color: #dee2e6 !important;
  }

  /* Smooth scrollbar for overflow menu */
  .overflow-tab-item {
    transition: background-color 0.15s;
  }
</style>
