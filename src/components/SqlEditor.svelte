<script>
  import { onMount, onDestroy } from "svelte";
  import { EditorView, basicSetup } from "codemirror";
  import { sql } from "@codemirror/lang-sql";
  import { activeConnection } from "../stores/connections";
  import { tabDataStore } from "../stores/tabData";
  import { executeQuery } from "../utils/tauri";

  export let tabId;

  let editorContainer;
  let editorView;
  let executing = false;

  $: tabData = $tabDataStore[tabId] || {
    queryText: "SELECT * FROM table_name LIMIT 100;",
  };

  onMount(() => {
    editorView = new EditorView({
      doc: tabData.queryText || "SELECT * FROM table_name LIMIT 100;",
      extensions: [
        basicSetup,
        sql(),
        EditorView.lineWrapping,
        EditorView.theme({
          "&": {
            backgroundColor: "#ffffff",
            color: "#333333",
            height: "100%",
          },
          ".cm-content": {
            fontFamily: "'Consolas', 'Monaco', 'Courier New', monospace",
            fontSize: "13px",
            padding: "8px 0",
          },
          ".cm-line": {
            padding: "0 8px",
          },
          ".cm-gutters": {
            backgroundColor: "#f5f5f5",
            color: "#999999",
            border: "none",
            borderRight: "1px solid #e0e0e0",
          },
          ".cm-activeLineGutter": {
            backgroundColor: "#e3f2fd",
          },
          ".cm-activeLine": {
            backgroundColor: "#fafafa",
          },
          ".cm-selectionBackground, ::selection": {
            backgroundColor: "#add6ff !important",
          },
          ".cm-cursor": {
            borderLeftColor: "#2196F3",
          },
        }),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            const text = update.state.doc.toString();
            tabDataStore.setQueryText(tabId, text);
          }
        }),
      ],
      parent: editorContainer,
    });
  });

  onDestroy(() => {
    if (editorView) {
      editorView.destroy();
    }
  });

  async function runQuery() {
    if (!$activeConnection) {
      alert("Please select a connection first");
      return;
    }

    const query = editorView.state.doc.toString();
    if (!query.trim()) {
      alert("Please enter a query");
      return;
    }

    executing = true;
    try {
      const result = await executeQuery($activeConnection, query);
      tabDataStore.setQueryResult(tabId, result);
      tabDataStore.setExecutedQuery(tabId, query);
    } catch (error) {
      alert("Query execution failed: " + error);
      console.error(error);
    }
    executing = false;
  }

  function clearEditor() {
    if (editorView) {
      editorView.dispatch({
        changes: { from: 0, to: editorView.state.doc.length, insert: "" },
      });
    }
  }
</script>

<div class="sql-editor-container h-100 d-flex flex-column">
  <div
    class="d-flex align-items-center justify-content-between bg-light border-bottom px-2 py-2"
    style="min-height: 40px;"
  >
    <div class="btn-group btn-group-sm">
      <button
        class="btn btn-success d-flex align-items-center gap-1"
        on:click={runQuery}
        disabled={executing || !$activeConnection}
        title="Execute SQL (Ctrl+Enter)"
      >
        <i class="fas fa-play"></i>
        <span>{executing ? "Executing..." : "Execute"}</span>
      </button>
      <button
        class="btn btn-outline-secondary d-flex align-items-center gap-1"
        on:click={clearEditor}
        title="Clear editor"
      >
        <i class="fas fa-eraser"></i>
      </button>
    </div>

    {#if !$activeConnection}
      <span
        class="alert alert-warning d-flex align-items-center gap-1 p-1 px-2 m-0"
        style="font-size: 11px;"
      >
        <i class="fas fa-exclamation-triangle"></i>
        <span>No connection selected</span>
      </span>
    {/if}
  </div>

  <div
    bind:this={editorContainer}
    class="flex-grow-1"
    style="overflow: auto;"
  ></div>
</div>

<style>
  .sql-editor-container :global(.cm-editor) {
    height: 100%;
  }

  .sql-editor-container :global(.cm-scroller) {
    overflow: auto;
  }
</style>
