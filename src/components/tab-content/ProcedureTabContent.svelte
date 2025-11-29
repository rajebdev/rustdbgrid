<script>
  import { onMount, onDestroy } from "svelte";
  import { EditorView, basicSetup } from "codemirror";
  import { sql } from "@codemirror/lang-sql";
  import { oneDark } from "@codemirror/theme-one-dark";
  import { activeTheme } from "../../stores/theme";

  export let procedure;
  export let database;
  export let connection;

  let editorContainer;
  let editorView;
  let loading = true;
  let error = null;
  let procedureSource = "";

  $: isDark = $activeTheme === "dark";

  // Reload when procedure changes
  $: if (procedure && database) {
    loadProcedureSource();
  }

  onMount(async () => {
    await loadProcedureSource();
    initializeEditor();
  });

  onDestroy(() => {
    if (editorView) {
      editorView.destroy();
    }
  });

  async function loadProcedureSource() {
    loading = true;
    error = null;
    try {
      // Import tauri function
      const { invoke } = await import("@tauri-apps/api/core");

      // Query to get procedure source - different for each database type
      let query;
      let sourceColumn;

      if (connection.db_type === "MySQL") {
        // MySQL uses SHOW CREATE syntax
        if (procedure.procedure_type === "FUNCTION") {
          query = `SHOW CREATE FUNCTION \`${database.name}\`.\`${procedure.name}\``;
          sourceColumn = "Create Function";
        } else {
          query = `SHOW CREATE PROCEDURE \`${database.name}\`.\`${procedure.name}\``;
          sourceColumn = "Create Procedure";
        }
      } else if (connection.db_type === "PostgreSQL") {
        // PostgreSQL uses prosrc for the source code
        const schemaName = procedure.schema || "public";

        // Get the complete function/procedure definition
        // Use LIKE to handle function overloading (same name with different parameters)
        query = `SELECT 
                   CASE 
                     WHEN p.prokind = 'p' THEN 
                       'CREATE OR REPLACE PROCEDURE ' || n.nspname || '.' || p.proname || 
                       '(' || pg_get_function_arguments(p.oid) || ')' || E'\\n' ||
                       'LANGUAGE ' || l.lanname || E'\\n' ||
                       'AS $' || '$' || E'\\n' || p.prosrc || E'\\n' || '$' || '$;'
                     ELSE
                       pg_get_functiondef(p.oid)
                   END as source
                 FROM pg_proc p
                 JOIN pg_namespace n ON p.pronamespace = n.oid
                 JOIN pg_language l ON p.prolang = l.oid
                 WHERE n.nspname = '${schemaName}' 
                   AND p.proname = '${procedure.name}'
                 ORDER BY p.oid
                 LIMIT 1`;
        sourceColumn = "source";
      } else if (connection.db_type === "MSSQL") {
        // MSSQL uses OBJECT_DEFINITION with schema-qualified name
        // Format: schema.object_name (don't include database name in OBJECT_ID)
        const schemaName = procedure.schema || "dbo";
        const qualifiedName = `${schemaName}.${procedure.name}`;
        query = `USE [${database.name}]; SELECT OBJECT_DEFINITION(OBJECT_ID('${qualifiedName}')) as source`;
        sourceColumn = "source";
      } else {
        throw new Error(`Unsupported database type: ${connection.db_type}`);
      }

      console.log("Loading procedure source with query:", query);
      console.log("Procedure object:", procedure);
      console.log("Procedure schema:", procedure.schema);
      console.log("Procedure name:", procedure.name);
      console.log("Database:", database);
      console.log("Connection:", connection);

      const result = await invoke("execute_query", {
        config: connection,
        query: query,
      });

      console.log("Query result:", result);
      console.log("Looking for column:", sourceColumn);

      if (result.rows && result.rows.length > 0) {
        console.log("First row:", result.rows[0]);
        console.log("Column names:", result.columns);
        procedureSource =
          result.rows[0][sourceColumn] || "-- No source available";
      } else {
        procedureSource = "-- No source available";
      }
    } catch (err) {
      console.error("Failed to load procedure source:", err);
      error = err.toString();
      procedureSource = `-- Error loading procedure source:\n-- ${err}`;
    } finally {
      loading = false;
    }
  }

  function initializeEditor() {
    if (!editorContainer) return;

    const extensions = [
      basicSetup,
      sql(),
      EditorView.editable.of(false), // Read-only
      EditorView.lineWrapping,
    ];

    if (isDark) {
      extensions.push(oneDark);
    }

    editorView = new EditorView({
      doc: procedureSource,
      extensions,
      parent: editorContainer,
    });
  }

  // Update editor content when procedureSource changes
  $: if (editorView && !loading) {
    const transaction = editorView.state.update({
      changes: {
        from: 0,
        to: editorView.state.doc.length,
        insert: procedureSource,
      },
    });
    editorView.dispatch(transaction);
  }

  // Update theme by recreating editor
  $: if (editorView && editorContainer) {
    isDark; // dependency
    editorView.destroy();
    initializeEditor();
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

  .editor-container :global(.cm-editor) {
    height: 100%;
    font-size: 13px;
  }

  .editor-container :global(.cm-scroller) {
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
