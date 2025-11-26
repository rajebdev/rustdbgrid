<script>
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  function handleAction(action) {
    dispatch(action);
  }
</script>

<div class="toolbar">
  <button
    class="toolbar-btn"
    title="New Connection"
    on:click={() => handleAction("newConnection")}
  >
    <i class="fas fa-plus"></i>
  </button>
  <div class="toolbar-divider"></div>
  <button
    class="toolbar-btn"
    title="New SQL Script (Ctrl+N)"
    on:click={() => handleAction("newQuery")}
  >
    <i class="fas fa-file-code"></i>
  </button>
  <button
    class="toolbar-btn"
    title="Save (Ctrl+S)"
    on:click={() => handleAction("saveQuery")}
  >
    <i class="fas fa-save"></i>
  </button>
  <div class="toolbar-divider"></div>
  <button
    class="toolbar-btn"
    title="Execute SQL (F5 / Ctrl+Enter)"
    on:click={() => handleAction("execute")}
  >
    <i class="fas fa-play" style="color: #4caf50;"></i>
  </button>
  <button
    class="toolbar-btn"
    title="Execute Script (Ctrl+Shift+Enter)"
    on:click={() => handleAction("executeScript")}
  >
    <i class="fas fa-play-circle" style="color: #4caf50;"></i>
  </button>
  <button
    class="toolbar-btn"
    title="Stop"
    on:click={() => handleAction("stop")}
  >
    <i class="fas fa-stop" style="color: #f44336;"></i>
  </button>
  <div class="toolbar-divider"></div>
  <button
    class="toolbar-btn"
    title="Commit"
    on:click={() => handleAction("commit")}
  >
    <i class="fas fa-check"></i>
  </button>
  <button
    class="toolbar-btn"
    title="Rollback"
    on:click={() => handleAction("rollback")}
  >
    <i class="fas fa-undo"></i>
  </button>
  <div class="toolbar-divider"></div>
  <button
    class="toolbar-btn"
    title="Refresh (Shift+F5)"
    on:click={() => handleAction("refresh")}
  >
    <i class="fas fa-sync-alt"></i>
  </button>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 2px;
    background: var(--bg-toolbar);
    border-bottom: 1px solid var(--border-color);
    height: 36px;
    padding: 0 12px;
    user-select: none;
  }

  .toolbar-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    padding: 6px 10px;
    font-size: 16px;
    cursor: pointer;
    transition: all 0.15s;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 32px;
    height: 28px;
  }

  .toolbar-btn:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .toolbar-btn:active {
    background: var(--border-color);
    transform: translateY(1px);
  }

  .toolbar-divider {
    width: 1px;
    height: 24px;
    background: var(--border-color);
    margin: 0 6px;
  }
</style>
