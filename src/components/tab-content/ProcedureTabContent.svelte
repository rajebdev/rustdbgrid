<script>
  import { onMount, onDestroy } from "svelte";
  import * as monaco from "monaco-editor";
  import { activeTheme } from "../../stores/theme";
  import { getMonacoTheme } from "../../services/themeService";
  import { getDatabaseObject } from "../../utils/tauri";

  export let procedure;
  export let database;
  export let connection;

  let editorContainer;
  let editor;
  let loading = true;
  let error = null;
  let procedureSource = "";
  let currentTheme = null;
  let lastLoadedProcedure = null;
  let lastLoadedDatabase = null;

  // React to theme changes
  $: if ($activeTheme && $activeTheme !== currentTheme) {
    currentTheme = $activeTheme;
    if (editor) {
      const theme = getMonacoTheme($activeTheme);
      monaco.editor.setTheme(theme);
    }
  }

  // Reload when procedure or database actually changes (not just re-renders)
  $: if (
    procedure &&
    database &&
    (lastLoadedProcedure?.name !== procedure.name ||
      lastLoadedProcedure?.schema !== procedure.schema ||
      lastLoadedDatabase?.name !== database.name)
  ) {
    lastLoadedProcedure = { name: procedure.name, schema: procedure.schema };
    lastLoadedDatabase = { name: database.name };
    loadProcedureSource();
  }

  onMount(async () => {
    await loadProcedureSource();
    initializeEditor();
  });

  onDestroy(() => {
    if (editor) {
      editor.dispose();
    }
  });

  async function loadProcedureSource() {
    loading = true;
    error = null;
    try {
      // Get connection ID - handle both object and string formats
      const connectionId =
        typeof connection === "string" ? connection : connection?.id;

      if (!connectionId) {
        throw new Error("Connection ID is not available");
      }

      // Determine request type based on procedure type
      const requestType =
        procedure.procedure_type === "FUNCTION" ? "function" : "procedure";

      // Use unified getDatabaseObject helper function
      const result = await getDatabaseObject(
        connectionId,
        requestType,
        database.name,
        procedure.schema,
        procedure.name
      );

      if (result && result.source) {
        procedureSource = result.source;
      } else {
        procedureSource = "-- No source available";
      }
    } catch (err) {
      console.error("Failed to load procedure source:", err);
      error = err.toString();
      procedureSource = `-- Error loading procedure source:\n-- ${err}`;
    } finally {
      loading = false;
      if (editor) {
        editor.setValue(procedureSource);
      }
    }
  }

  function initializeEditor() {
    if (!editorContainer || loading) return;

    const theme = getMonacoTheme($activeTheme);

    editor = monaco.editor.create(editorContainer, {
      value: procedureSource,
      language: "sql",
      theme,
      readOnly: true,
      automaticLayout: true,
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      wordWrap: "on",
      fontSize: 13,
      fontFamily: "'Fira Code', 'Monaco', monospace",
      folding: true,
    });
  }

  async function copyToClipboard() {
    try {
      await navigator.clipboard.writeText(procedureSource);
      // Optional: show toast notification
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  }
</script>

<div class="procedure-viewer h-100 d-flex flex-column">
  <div class="procedure-header p-2 border-bottom">
    <div class="d-flex justify-content-between align-items-center">
      <h6 class="mb-0">
        <i class="fas fa-cog me-2"></i>
        {procedure.name}
        {#if procedure.procedure_type}
          <span
            class="badge {procedure.procedure_type === 'FUNCTION'
              ? 'bg-success'
              : 'bg-secondary'} ms-2"
          >
            {procedure.procedure_type}
          </span>
        {/if}
      </h6>
      <div class="btn-group btn-group-sm">
        <button
          class="btn btn-outline-secondary"
          on:click={copyToClipboard}
          title="Copy to clipboard"
          disabled={loading}
        >
          <i class="fas fa-copy"></i>
        </button>
        <button
          class="btn btn-outline-secondary"
          on:click={loadProcedureSource}
          title="Refresh"
          disabled={loading}
        >
          <i class="fas fa-sync-alt" class:fa-spin={loading}></i>
        </button>
      </div>
    </div>
  </div>

  <div class="procedure-content flex-grow-1 position-relative">
    {#if loading}
      <div
        class="loading-overlay d-flex justify-content-center align-items-center h-100"
      >
        <div class="text-center">
          <i class="fas fa-spinner fa-spin fa-2x mb-2"></i>
          <p>Loading procedure source...</p>
        </div>
      </div>
    {:else if error}
      <div class="error-message p-3">
        <div class="alert alert-danger">
          <i class="fas fa-exclamation-triangle me-2"></i>
          {error}
        </div>
      </div>
    {:else}
      <div bind:this={editorContainer} class="editor-container h-100"></div>
    {/if}
  </div>
</div>

<style>
  .procedure-viewer {
    background-color: var(--bg-primary);
  }

  .procedure-header {
    background: var(--bg-tertiary);
    border-color: var(--border-color) !important;
    color: var(--text-primary);
  }

  .procedure-header h6 {
    color: var(--text-primary);
  }

  .procedure-content {
    overflow: hidden;
  }

  .editor-container {
    width: 100%;
    height: 100%;
    overflow: auto;
  }

  .loading-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--bg-primary);
    color: var(--text-primary);
    z-index: 10;
  }

  .error-message {
    padding: 1rem;
  }

  h6 {
    font-size: 14px;
    font-weight: 600;
  }
</style>
