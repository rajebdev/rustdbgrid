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

<div
  class="position-fixed bg-body border shadow-sm"
  style="left: {x}px; top: {y}px; min-width: 220px; z-index: 99999; border-radius: 4px; overflow: hidden;"
>
  <button
    class="context-menu-item d-flex align-items-center gap-2 w-100 border-0 bg-transparent px-3 py-2 text-start"
    on:click={close}
  >
    <span class="flex-grow-1" style="font-size: 13px;">Close</span>
  </button>

  <button
    class="context-menu-item d-flex align-items-center gap-2 w-100 border-0 bg-transparent px-3 py-2 text-start"
    on:click={closeOthers}
    disabled={!canCloseOthers}
  >
    <span class="flex-grow-1" style="font-size: 13px;">Close Others</span>
  </button>

  <button
    class="context-menu-item d-flex align-items-center gap-2 w-100 border-0 bg-transparent px-3 py-2 text-start"
    on:click={closeToLeft}
    disabled={!canCloseLeft}
  >
    <span class="flex-grow-1" style="font-size: 13px;"
      >Close Tabs to the Left</span
    >
  </button>

  <button
    class="context-menu-item d-flex align-items-center gap-2 w-100 border-0 bg-transparent px-3 py-2 text-start"
    on:click={closeToRight}
    disabled={!canCloseRight}
  >
    <span class="flex-grow-1" style="font-size: 13px;"
      >Close Tabs to the Right</span
    >
  </button>

  <button
    class="context-menu-item d-flex align-items-center gap-2 w-100 border-0 bg-transparent px-3 py-2 text-start"
    on:click={closeAll}
  >
    <span class="flex-grow-1" style="font-size: 13px;">Close All</span>
  </button>

  <div class="border-top my-1"></div>

  <button
    class="context-menu-item d-flex align-items-center gap-2 w-100 border-0 bg-transparent px-3 py-2 text-start"
    on:click={detach}
    disabled={true}
    title="TODO: Not yet implemented"
  >
    <span class="flex-grow-1" style="font-size: 13px;"
      >Detach <span class="text-muted" style="font-size: 11px;">(TODO)</span
      ></span
    >
  </button>

  {#if tabType === "table"}
    <div class="border-top my-1"></div>
    <button
      class="context-menu-item d-flex align-items-center gap-2 w-100 border-0 bg-transparent px-3 py-2 text-start"
      on:click={copyObjectName}
    >
      <span class="flex-grow-1" style="font-size: 13px;">Copy Object Name</span>
    </button>

    <button
      class="context-menu-item d-flex align-items-center gap-2 w-100 border-0 bg-transparent px-3 py-2 text-start"
      on:click={addBookmark}
      disabled={true}
      title="TODO: Not yet implemented"
    >
      <i class="fas fa-star text-warning" style="font-size: 12px; width: 16px;"
      ></i>
      <span class="flex-grow-1" style="font-size: 13px;"
        >Add Bookmark <span class="text-muted" style="font-size: 11px;"
          >(TODO)</span
        ></span
      >
      <span class="text-secondary" style="font-size: 11px;"
        >Ctrl+Alt+Shift+D</span
      >
    </button>
  {/if}

  {#if tabType === "query"}
    <div class="border-top my-1"></div>

    <button
      class="context-menu-item d-flex align-items-center gap-2 w-100 border-0 bg-transparent px-3 py-2 text-start"
      on:click={newScript}
    >
      <i class="fas fa-file-code" style="font-size: 12px; width: 16px;"></i>
      <span class="flex-grow-1" style="font-size: 13px;">New Script</span>
    </button>

    <button
      class="context-menu-item d-flex align-items-center gap-2 w-100 border-0 bg-transparent px-3 py-2 text-start"
      on:click={revealInExplorer}
      disabled={canDisableFilePathOps}
      title={canDisableFilePathOps ? "Save the query first" : ""}
      style={canDisableFilePathOps ? "opacity: 0.5; cursor: not-allowed;" : ""}
    >
      <i class="fas fa-folder-open" style="font-size: 12px; width: 16px;"></i>
      <span class="flex-grow-1" style="font-size: 13px;"
        >Reveal in Explorer</span
      >
    </button>

    <button
      class="context-menu-item d-flex align-items-center gap-2 w-100 border-0 bg-transparent px-3 py-2 text-start"
      on:click={copyFilePath}
      disabled={canDisableFilePathOps}
      title={canDisableFilePathOps ? "Save the query first" : ""}
      style={canDisableFilePathOps ? "opacity: 0.5; cursor: not-allowed;" : ""}
    >
      <i class="fas fa-copy" style="font-size: 12px; width: 16px;"></i>
      <span class="flex-grow-1" style="font-size: 13px;">Copy File Path</span>
    </button>

    <div class="border-top my-1"></div>

    <button
      class="context-menu-item d-flex align-items-center gap-2 w-100 border-0 bg-transparent px-3 py-2 text-start"
      on:click={renameFile}
    >
      <i class="fas fa-edit" style="font-size: 12px; width: 16px;"></i>
      <span class="flex-grow-1" style="font-size: 13px;">Rename File</span>
    </button>

    <button
      class="context-menu-item d-flex align-items-center gap-2 w-100 border-0 bg-transparent px-3 py-2 text-start"
      on:click={deleteScript}
    >
      <i class="fas fa-trash text-danger" style="font-size: 12px; width: 16px;"
      ></i>
      <span class="flex-grow-1 text-danger" style="font-size: 13px;"
        >Delete This Script</span
      >
    </button>
  {/if}
</div>

<style>
  .context-menu-item:hover:not(:disabled) {
    background-color: var(--hover-bg);
  }

  .context-menu-item:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .context-menu-item {
    cursor: pointer;
    transition: background-color 0.1s;
    color: var(--text-primary);
  }
</style>
