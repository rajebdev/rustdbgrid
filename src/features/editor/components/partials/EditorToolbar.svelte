<script>
  export let executing = false;
  export let selectedConn = null;
  export let selectedDb = null;

  export let onExecute = null;
  export let onExecuteNewTab = null;
</script>

<div
  class="editor-actions-sidebar p-3 d-flex flex-column align-items-center py-2 gap-1"
>
  <button
    class="btn-action btn-execute"
    on:click={() => onExecute?.()}
    disabled={executing || !selectedConn}
    title="Execute (Ctrl+Enter)"
  >
    <i class="fas fa-play"></i>
  </button>
  <button
    class="btn-action btn-execute-new"
    on:click={() => onExecuteNewTab?.()}
    disabled={executing || !selectedConn || !selectedDb}
    title="Execute in New Tab (Ctrl+Shift+Enter)"
  >
    <i class="fas fa-play"></i>
    <i class="fas fa-plus icon-plus"></i>
  </button>

  {#if executing}
    <div class="executing-indicator">
      <i class="fas fa-spinner fa-spin"></i>
    </div>
  {/if}
</div>

<style>
  .editor-actions-sidebar {
    background: var(--bg-tertiary);
    border-right: 1px solid var(--border-color);
    padding: 3px;
    width: 24px;
    flex-shrink: 0;
  }

  .btn-action {
    width: 20px;
    height: 20px;
    border: none;
    border-radius: 3px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 9px;
    transition: all 0.15s ease;
    position: relative;
  }

  .btn-action:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-execute {
    background: var(--accent-green, #22c55e);
    color: white;
  }

  .btn-execute:hover:not(:disabled) {
    background: var(--accent-green-hover, #16a34a);
    transform: scale(1.05);
  }

  .btn-execute-new {
    background: var(--accent-blue, #3b82f6);
    color: white;
  }

  .btn-execute-new:hover:not(:disabled) {
    background: var(--accent-blue-hover, #2563eb);
    transform: scale(1.05);
  }

  .btn-execute-new .icon-plus {
    font-size: 5px;
    position: absolute;
    bottom: 1px;
    right: 1px;
  }

  .executing-indicator {
    color: var(--accent-blue);
    font-size: 12px;
    margin-top: 4px;
  }
</style>
