<script>
  export let item = {};
  export let onClick = () => {};
</script>

{#if item.type === "divider"}
  <div class="context-menu-divider"></div>
{:else}
  <button
    class="context-menu-item"
    class:danger={item.danger}
    disabled={item.disabled}
    on:click={onClick}
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

<style>
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
    font-size: 12px;
  }

  .context-menu-item:hover:not(:disabled) {
    background: var(--hover-bg);
  }

  .context-menu-item:disabled {
    color: var(--text-muted);
    cursor: not-allowed;
    opacity: 0.6;
  }

  .context-menu-item.danger {
    color: var(--accent-red);
  }

  .context-menu-item.danger:hover:not(:disabled) {
    background: var(--accent-red-light);
  }

  .context-menu-item i:first-child {
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
