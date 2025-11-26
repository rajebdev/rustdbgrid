<script>
  import SqlEditor from "../common/SqlEditor.svelte";
  import DataGrid from "../common/DataGrid.svelte";
  import { activeConnection } from "../../stores/connections";

  export let tabId;
  export let currentTabData;
  export let editorHeight = 300;
  export let isResizingEditor = false;

  function handleEditorMouseDown(event) {
    event.preventDefault();
    window.dispatchEvent(
      new CustomEvent("start-editor-resize", { detail: { event } })
    );
  }

  function handleEditorMouseMove(event) {
    if (isResizingEditor) {
      window.dispatchEvent(
        new CustomEvent("editor-resize-move", { detail: { event } })
      );
    }
  }
</script>

<div
  class="d-flex flex-column h-100"
  class:resizing={isResizingEditor}
  on:mousemove={handleEditorMouseMove}
  role="presentation"
>
  <div
    class="border-bottom-2 border-secondary position-relative"
    style="height: {currentTabData?.queryResult
      ? `${editorHeight}px`
      : '100%'}; flex-shrink: 0; overflow: hidden;"
  >
    <SqlEditor {tabId} />
  </div>

  {#if currentTabData?.queryResult}
    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      class="bg-body-tertiary border-top border-bottom editor-resize-handle"
      style="height: 6px; cursor: row-resize;"
      on:mousedown={handleEditorMouseDown}
      role="separator"
      aria-orientation="horizontal"
      aria-label="Resize editor"
    ></div>
    <div class="flex-grow-1 overflow-hidden">
      <DataGrid
        data={currentTabData.queryResult}
        {tabId}
        executedQuery={currentTabData?.executedQuery || ""}
        connection={$activeConnection}
      />
    </div>
  {/if}
</div>

<style>
  .editor-resize-handle {
    display: block;
    transition: background-color 0.2s;
  }

  .editor-resize-handle:hover {
    background-color: #0d6efd !important;
  }

  .editor-resize-handle:focus {
    outline: 2px solid #0d6efd;
    outline-offset: -2px;
  }

  .resizing {
    cursor: row-resize !important;
  }

  .resizing :global(*) {
    user-select: none !important;
    pointer-events: none !important;
    cursor: row-resize !important;
  }
</style>
