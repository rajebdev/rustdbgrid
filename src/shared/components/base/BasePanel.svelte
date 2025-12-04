<script>
  export let title = "";
  export let icon = "";
  export let showToolbar = true;
</script>

<div class="base-panel">
  <div class="panel-header">
    <h6 class="mb-0">
      {#if icon}
        <i class={icon}></i>
      {/if}
      {title}
    </h6>
  </div>

  {#if showToolbar}
    <div class="panel-toolbar">
      <slot name="toolbar" />
    </div>
  {/if}

  <div class="panel-content">
    <slot />
  </div>
</div>

<style>
  .base-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
  }

  .panel-header {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-tertiary);
    flex-shrink: 0;
  }

  .panel-header h6 {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .panel-toolbar {
    display: flex;
    gap: 4px;
    padding: 8px;
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }
</style>
