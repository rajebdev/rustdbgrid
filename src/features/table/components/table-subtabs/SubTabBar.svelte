<script>
  export let activeSubTab = "properties";
  export let onTabChange;

  const tabs = [
    { id: "properties", label: "Properties", icon: "fas fa-list" },
    { id: "data", label: "Data", icon: "fas fa-table" },
    { id: "diagram", label: "Diagram", icon: "fas fa-project-diagram" },
  ];

  function handleTabClick(tabId) {
    activeSubTab = tabId;
    if (onTabChange) {
      onTabChange(tabId);
    }
  }
</script>

<div class="subtab-bar">
  {#each tabs as tab}
    <button
      class="subtab-btn"
      class:active={activeSubTab === tab.id}
      on:click={() => handleTabClick(tab.id)}
    >
      <i class="{tab.icon} tab-icon"></i>
      <span class="tab-label">{tab.label}</span>
    </button>
  {/each}
</div>

<style>
  .subtab-bar {
    display: flex;
    gap: 0;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    padding: 0;
    height: 32px;
    align-items: flex-end;
  }

  .subtab-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 14px;
    background: var(--bg-tertiary);
    border: none;
    border-top-left-radius: 6px;
    border-top-right-radius: 6px;
    border: 1px solid transparent;
    border-bottom: none;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 400;
    cursor: pointer;
    transition: all 0.15s ease;
    height: 28px;
    margin-right: 2px;
    position: relative;
  }

  .subtab-btn:hover:not(.active) {
    background: var(--bg-secondary);
    color: var(--text-primary);
  }

  .subtab-btn.active {
    background: var(--bg-primary);
    color: var(--text-primary);
    border-color: var(--border-color);
    border-bottom-color: var(--bg-primary);
    font-weight: 400;
    z-index: 1;
  }

  .tab-icon {
    font-size: 11px;
    opacity: 0.8;
  }

  .subtab-btn.active .tab-icon {
    opacity: 1;
  }

  .tab-label {
    white-space: nowrap;
  }
</style>
