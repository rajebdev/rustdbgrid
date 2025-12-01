<script>
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  export let x = 0;
  export let y = 0;
  export let canCloseLeft = false;
  export let canCloseRight = false;
  export let canCloseOthers = false;
  export let tabType = null; // "query", "table", "procedure", etc.
  export let tab = null; // Full tab data to check if it's saved

  let canDisableFilePathOps = false;

  // Reactive statement to ensure proper reactivity
  // Disable only if query tab has no filePath at all
  $: isQueryTab = tabType === "query";
  $: canDisableFilePathOps = isQueryTab && !tab?.filePath;

  function close() {
    dispatch("close");
  }

  function closeOthers() {
    dispatch("closeOthers");
  }

  function closeToLeft() {
    dispatch("closeToLeft");
  }

  function closeToRight() {
    dispatch("closeToRight");
  }

  function closeAll() {
    dispatch("closeAll");
  }

  function detach() {
    dispatch("detach");
  }

  function copyObjectName() {
    dispatch("copyObjectName");
  }

  function addBookmark() {
    dispatch("addBookmark");
  }

  function newScript() {
    dispatch("newScript");
  }

  function revealInExplorer() {
    // Only dispatch if file is saved
    if (!canDisableFilePathOps) {
      dispatch("revealInExplorer");
    }
  }

  function copyFilePath() {
    // Only dispatch if file is saved
    if (!canDisableFilePathOps) {
      dispatch("copyFilePath");
    }
  }

  function deleteScript() {
    dispatch("deleteScript");
  }

  function renameFile() {
    dispatch("renameFile");
  }
</script>

<div class="context-menu" style="left: {x}px; top: {y}px;">
  <div class="context-menu-section">
    <button class="context-menu-item" on:click={close}>
      <i class="fas fa-times"></i>
      <span>Close</span>
    </button>

    <button
      class="context-menu-item"
      on:click={closeOthers}
      disabled={!canCloseOthers}
    >
      <i class="fas fa-times-circle"></i>
      <span>Close Others</span>
    </button>

    <button
      class="context-menu-item"
      on:click={closeToLeft}
      disabled={!canCloseLeft}
    >
      <i class="fas fa-arrow-left"></i>
      <span>Close Tabs to the Left</span>
    </button>

    <button
      class="context-menu-item"
      on:click={closeToRight}
      disabled={!canCloseRight}
    >
      <i class="fas fa-arrow-right"></i>
      <span>Close Tabs to the Right</span>
    </button>

    <button class="context-menu-item" on:click={closeAll}>
      <i class="fas fa-window-close"></i>
      <span>Close All</span>
    </button>
  </div>

  <div class="context-menu-divider"></div>

  <div class="context-menu-section">
    <button class="context-menu-item" on:click={detach} disabled>
      <i class="fas fa-external-link-alt"></i>
      <span>Detach <span class="text-muted">(TODO)</span></span>
    </button>
  </div>

  {#if tabType === "table"}
    <div class="context-menu-divider"></div>

    <div class="context-menu-section">
      <button class="context-menu-item" on:click={copyObjectName}>
        <i class="fas fa-copy"></i>
        <span>Copy Object Name</span>
      </button>

      <button class="context-menu-item" on:click={addBookmark} disabled>
        <i class="fas fa-star text-warning"></i>
        <span>Add Bookmark <span class="text-muted">(TODO)</span></span>
        <kbd>Ctrl+Alt+Shift+D</kbd>
      </button>
    </div>
  {/if}

  {#if tabType === "query"}
    <div class="context-menu-divider"></div>

    <div class="context-menu-section">
      <button class="context-menu-item" on:click={newScript}>
        <i class="fas fa-file-code"></i>
        <span>New Script</span>
      </button>

      <button
        class="context-menu-item"
        on:click={revealInExplorer}
        disabled={canDisableFilePathOps}
        title={canDisableFilePathOps ? "Save the query first" : ""}
      >
        <i class="fas fa-folder-open"></i>
        <span>Reveal in Explorer</span>
      </button>

      <button
        class="context-menu-item"
        on:click={copyFilePath}
        disabled={canDisableFilePathOps}
        title={canDisableFilePathOps ? "Save the query first" : ""}
      >
        <i class="fas fa-copy"></i>
        <span>Copy File Path</span>
      </button>
    </div>

    <div class="context-menu-divider"></div>

    <div class="context-menu-section">
      <button class="context-menu-item" on:click={renameFile}>
        <i class="fas fa-edit"></i>
        <span>Rename File</span>
      </button>

      <button class="context-menu-item text-danger" on:click={deleteScript}>
        <i class="fas fa-trash"></i>
        <span>Delete This Script</span>
      </button>
    </div>
  {/if}
</div>

<style>
  .context-menu {
    position: fixed;
    background: var(--bg-dropdown);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    box-shadow: var(--shadow-dropdown);
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

  .context-menu-item:disabled {
    color: var(--text-muted);
    cursor: not-allowed;
    opacity: 0.6;
  }

  .context-menu-item.text-danger {
    color: var(--danger-color);
  }

  .context-menu-item.text-danger:hover:not(:disabled) {
    background: var(--danger-bg-subtle);
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

  .context-menu-item .text-muted {
    font-size: 10px;
    color: var(--text-muted);
  }

  .context-menu-divider {
    height: 1px;
    background: var(--border-color);
    margin: 4px 0;
  }
</style>
