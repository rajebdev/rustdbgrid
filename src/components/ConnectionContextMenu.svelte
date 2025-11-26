<script>
  import { createEventDispatcher } from "svelte";

  export let x = 0;
  export let y = 0;
  export let connection = null;
  export let isConnected = false;

  const dispatch = createEventDispatcher();

  function handleEdit() {
    dispatch("edit", connection);
  }

  function handleDelete() {
    dispatch("delete", connection);
  }

  function handleRefresh() {
    dispatch("refresh", connection);
  }

  function handleConnect() {
    dispatch("connect", connection);
  }

  function handleDisconnect() {
    dispatch("disconnect", connection);
  }

  function handleCopy() {
    dispatch("copy", connection);
  }
</script>

<div
  class="context-menu"
  style="left: {x}px; top: {y}px;"
  role="menu"
  tabindex="-1"
  on:click|stopPropagation
  on:keydown|stopPropagation
>
  <div class="context-menu-section">
    <button class="context-menu-item" disabled>
      <i class="fas fa-code"></i>
      <span>SQL Editor</span>
      <kbd>F3</kbd>
    </button>
    <button class="context-menu-item context-menu-item-with-arrow" disabled>
      <i class="fas fa-plus"></i>
      <span>Create</span>
      <i class="fas fa-chevron-right ms-auto"></i>
    </button>
  </div>

  <div class="context-menu-divider"></div>

  <div class="context-menu-section">
    <button class="context-menu-item" on:click={handleEdit}>
      <i class="fas fa-edit"></i>
      <span>Edit Connection</span>
      <kbd>F4</kbd>
    </button>
    <button class="context-menu-item context-menu-item-with-arrow" disabled>
      <i class="fas fa-eye"></i>
      <span>Connection view</span>
      <i class="fas fa-chevron-right ms-auto"></i>
    </button>
    <button class="context-menu-item" disabled>
      <i class="fas fa-folder-open"></i>
      <span>Browse from here</span>
    </button>
  </div>

  <div class="context-menu-divider"></div>

  <div class="context-menu-section">
    <button
      class="context-menu-item"
      on:click={handleConnect}
      disabled={isConnected}
    >
      <i class="fas fa-plug"></i>
      <span>Connect</span>
    </button>
    <button
      class="context-menu-item"
      on:click={handleRefresh}
      disabled={!isConnected}
    >
      <i class="fas fa-sync"></i>
      <span>Invalidate/Reconnect</span>
    </button>
    <button
      class="context-menu-item"
      on:click={handleDisconnect}
      disabled={!isConnected}
    >
      <i class="fas fa-unlink"></i>
      <span>Disconnect</span>
    </button>
  </div>

  <div class="context-menu-divider"></div>

  <div class="context-menu-section">
    <button class="context-menu-item context-menu-item-with-arrow" disabled>
      <i class="fas fa-exchange-alt"></i>
      <span>Compare/Migrate</span>
      <i class="fas fa-chevron-right ms-auto"></i>
    </button>
    <button class="context-menu-item context-menu-item-with-arrow" disabled>
      <i class="fas fa-tools"></i>
      <span>Tools</span>
      <i class="fas fa-chevron-right ms-auto"></i>
    </button>
  </div>

  <div class="context-menu-divider"></div>

  <div class="context-menu-section">
    <button class="context-menu-item" on:click={handleCopy}>
      <i class="fas fa-copy"></i>
      <span>Copy</span>
      <kbd>Ctrl+C</kbd>
    </button>
    <button class="context-menu-item" disabled>
      <i class="fas fa-paste"></i>
      <span>Paste</span>
      <kbd>Ctrl+V</kbd>
    </button>
    <button class="context-menu-item" disabled>
      <i class="fas fa-clone"></i>
      <span>Copy Advanced Info</span>
      <kbd>Ctrl+Shift+C</kbd>
    </button>
  </div>

  <div class="context-menu-divider"></div>

  <div class="context-menu-section">
    <button class="context-menu-item text-danger" on:click={handleDelete}>
      <i class="fas fa-trash"></i>
      <span>Delete</span>
      <kbd>Delete</kbd>
    </button>
    <button class="context-menu-item" disabled>
      <i class="fas fa-pen"></i>
      <span>Rename</span>
      <kbd>F2</kbd>
    </button>
  </div>

  <div class="context-menu-divider"></div>

  <div class="context-menu-section">
    <button class="context-menu-item" on:click={handleRefresh}>
      <i class="fas fa-redo"></i>
      <span>Refresh</span>
      <kbd>F5</kbd>
    </button>
  </div>
</div>

<style>
  .context-menu {
    position: fixed;
    background: white;
    border: 1px solid #dee2e6;
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    z-index: 10000;
    min-width: 240px;
    padding: 4px;
    font-size: 12px;
  }

  .context-menu-section {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .context-menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: transparent;
    border: none;
    color: #212529;
    cursor: pointer;
    text-align: left;
    border-radius: 4px;
    transition: background-color 0.15s;
    width: 100%;
    white-space: nowrap;
  }

  .context-menu-item:hover:not(:disabled) {
    background: #f8f9fa;
  }

  .context-menu-item:disabled {
    color: #adb5bd;
    cursor: not-allowed;
    opacity: 0.6;
  }

  .context-menu-item.text-danger {
    color: #dc3545;
  }

  .context-menu-item.text-danger:hover:not(:disabled) {
    background: #fff5f5;
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
    background: #e9ecef;
    border-radius: 3px;
    color: #6c757d;
    font-family: monospace;
    margin-left: auto;
  }

  .context-menu-item-with-arrow {
    padding-right: 8px;
  }

  .context-menu-item-with-arrow i.fa-chevron-right {
    font-size: 10px;
    color: #adb5bd;
    margin-left: auto;
  }

  .context-menu-divider {
    height: 1px;
    background: #dee2e6;
    margin: 4px 0;
  }
</style>
