<script>
  import { onMount, onDestroy } from "svelte";
  import { EditorView, basicSetup } from "codemirror";
  import { EditorState } from "@codemirror/state";
  import { drawSelection, highlightActiveLineGutter } from "@codemirror/view";
  import { sql } from "@codemirror/lang-sql";
  import { autocompletion } from "@codemirror/autocomplete";
  import {
    activeConnection,
    connections,
    selectedDatabase,
  } from "../../stores/connections";
  import { tabDataStore } from "../../stores/tabData";
  import { getDefaultQuery } from "../../utils/defaultQueries";
  import {
    executeQuery,
    getDatabases,
    getTables,
    getTableSchema,
  } from "../../utils/tauri";

  export let tabId;

  let editorContainer;
  let editorView;
  let executing = false;
  let databases = [];
  let selectedConn = null;
  let selectedDb = null;
  let loadingDatabases = false;
  let tables = [];
  let schema = {};

  // Subscribe to active connection changes
  $: if ($activeConnection) {
    selectedConn = $activeConnection;
    loadDatabases();
  }

  // Subscribe to selected database changes
  $: if ($selectedDatabase) {
    selectedDb = $selectedDatabase;
  }

  $: tabData = $tabDataStore[tabId] || {
    queryText: getDefaultQuery(selectedConn?.db_type || "MySQL"),
  };

  async function loadDatabases() {
    if (!selectedConn) return;

    loadingDatabases = true;
    try {
      databases = await getDatabases(selectedConn);
      if (databases.length > 0 && !selectedDb) {
        selectedDb = databases[0].name;
        selectedDatabase.set(selectedDb);
      }
    } catch (error) {
      console.error("Failed to load databases:", error);
      databases = [];
    }
    loadingDatabases = false;
  }

  async function loadTablesAndSchema() {
    if (!selectedConn || !selectedDb) return;

    try {
      tables = await getTables(selectedConn, selectedDb);

      // Build schema for autocomplete
      const newSchema = {};
      for (const table of tables) {
        try {
          const tableSchema = await getTableSchema(
            selectedConn,
            selectedDb,
            table.name
          );
          newSchema[table.name] = tableSchema.columns.map((col) => col.name);
        } catch (error) {
          console.error(
            `Failed to load schema for table ${table.name}:`,
            error
          );
        }
      }
      schema = newSchema;

      // Update editor with new schema
      if (editorView) {
        updateEditorExtensions();
      }
    } catch (error) {
      console.error("Failed to load tables:", error);
    }
  }

  function createAutocompletions() {
    // SQL Keywords for autocomplete (UPPERCASE)
    const sqlKeywords = [
      "SELECT",
      "FROM",
      "WHERE",
      "INSERT",
      "UPDATE",
      "DELETE",
      "JOIN",
      "INNER",
      "LEFT",
      "RIGHT",
      "OUTER",
      "ON",
      "AND",
      "OR",
      "NOT",
      "IN",
      "EXISTS",
      "BETWEEN",
      "LIKE",
      "IS",
      "NULL",
      "ORDER",
      "BY",
      "GROUP",
      "HAVING",
      "LIMIT",
      "OFFSET",
      "AS",
      "DISTINCT",
      "ALL",
      "UNION",
      "INTERSECT",
      "EXCEPT",
      "CREATE",
      "ALTER",
      "DROP",
      "TABLE",
      "DATABASE",
      "INDEX",
      "VIEW",
      "PRIMARY",
      "KEY",
      "FOREIGN",
      "REFERENCES",
      "CONSTRAINT",
      "UNIQUE",
      "CHECK",
      "DEFAULT",
      "CASCADE",
      "SET",
      "VALUES",
      "INTO",
      "CASE",
      "WHEN",
      "THEN",
      "ELSE",
      "END",
      "COUNT",
      "SUM",
      "AVG",
      "MIN",
      "MAX",
      "CAST",
      "COALESCE",
      "NULLIF",
      "IFNULL",
      "CONCAT",
      "SUBSTRING",
      "TRIM",
      "UPPER",
      "LOWER",
      "LENGTH",
      "ROUND",
      "FLOOR",
      "CEIL",
      "ABS",
      "NOW",
      "CURDATE",
      "CURTIME",
      "DATE",
      "TIME",
      "DATETIME",
      "TIMESTAMP",
    ].map((keyword) => ({
      label: keyword,
      type: "keyword",
      boost: 3,
      apply: keyword,
    }));

    // Create custom SQL completions based on schema (keep original case)
    const tableCompletions = Object.keys(schema).map((tableName) => ({
      label: tableName,
      type: "type",
      boost: 2,
      apply: tableName,
    }));

    const columnCompletions = [];
    for (const [tableName, columns] of Object.entries(schema)) {
      for (const columnName of columns) {
        columnCompletions.push({
          label: `${tableName}.${columnName}`,
          type: "property",
          boost: 1,
          apply: `${tableName}.${columnName}`,
        });
        columnCompletions.push({
          label: columnName,
          type: "property",
          boost: 0,
          apply: columnName,
        });
      }
    }

    return function sqlAutoComplete(context) {
      const word = context.matchBefore(/\w*/);
      if (!word || (word.from === word.to && !context.explicit)) {
        return null;
      }

      return {
        from: word.from,
        options: [...sqlKeywords, ...tableCompletions, ...columnCompletions],
      };
    };
  }

  function updateEditorExtensions() {
    if (!editorView) return;

    const extensions = [
      basicSetup,
      drawSelection(),
      highlightActiveLineGutter(),
      sql(),
      autocompletion({
        override: [createAutocompletions()],
      }),
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
          caretColor: "#2196F3",
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
        ".cm-selectionBackground": {
          backgroundColor: "#add6ff !important",
        },
        "&.cm-focused .cm-selectionBackground": {
          backgroundColor: "#add6ff !important",
        },
        "::selection": {
          backgroundColor: "#add6ff",
        },
        ".cm-cursor, .cm-dropCursor": {
          borderLeftColor: "#2196F3",
          borderLeftWidth: "2px",
        },
        ".cm-tooltip-autocomplete": {
          backgroundColor: "#ffffff",
          border: "1px solid #ccc",
          boxShadow: "0 2px 8px rgba(0,0,0,0.15)",
        },
        ".cm-tooltip-autocomplete > ul > li[aria-selected]": {
          backgroundColor: "#0d6efd",
          color: "#ffffff",
        },
      }),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          const text = update.state.doc.toString();
          tabDataStore.setQueryText(tabId, text);
        }
      }),
    ];

    // Recreate the editor view with new extensions
    const currentDoc = editorView.state.doc.toString();
    editorView.destroy();

    editorView = new EditorView({
      doc: currentDoc,
      extensions,
      parent: editorContainer,
    });
  }

  onMount(() => {
    editorView = new EditorView({
      doc:
        tabData.queryText || getDefaultQuery(selectedConn?.db_type || "MySQL"),
      extensions: [
        basicSetup,
        drawSelection(),
        highlightActiveLineGutter(),
        sql(),
        autocompletion({
          override: [createAutocompletions()],
        }),
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
            caretColor: "#2196F3",
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
          ".cm-selectionBackground": {
            backgroundColor: "#add6ff !important",
          },
          "&.cm-focused .cm-selectionBackground": {
            backgroundColor: "#add6ff !important",
          },
          "::selection": {
            backgroundColor: "#add6ff",
          },
          ".cm-cursor, .cm-dropCursor": {
            borderLeftColor: "#2196F3",
            borderLeftWidth: "2px",
          },
          ".cm-tooltip-autocomplete": {
            backgroundColor: "#ffffff",
            border: "1px solid #ccc",
            boxShadow: "0 2px 8px rgba(0,0,0,0.15)",
          },
          ".cm-tooltip-autocomplete > ul > li[aria-selected]": {
            backgroundColor: "#0d6efd",
            color: "#ffffff",
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

    // Load databases if connection is already selected
    if (selectedConn) {
      loadDatabases();
    }

    // Listen for custom events from menu bar
    const handleExecuteQuery = (event) => {
      if (event.detail.tabId === tabId) {
        runQuery();
      }
    };

    const handleExecuteScript = (event) => {
      if (event.detail.tabId === tabId) {
        runQuery(); // For now, execute script is the same as execute
      }
    };

    const handleUndo = (event) => {
      if (event.detail.tabId === tabId && editorView) {
        import("@codemirror/commands").then(({ undo }) => {
          undo(editorView);
        });
      }
    };

    const handleRedo = (event) => {
      if (event.detail.tabId === tabId && editorView) {
        import("@codemirror/commands").then(({ redo }) => {
          redo(editorView);
        });
      }
    };

    const handlePaste = async (event) => {
      if (event.detail.tabId === tabId && editorView && event.detail.text) {
        const selection = editorView.state.selection.main;
        editorView.dispatch({
          changes: {
            from: selection.from,
            to: selection.to,
            insert: event.detail.text,
          },
        });
      }
    };

    document.addEventListener("execute-query", handleExecuteQuery);
    document.addEventListener("execute-script", handleExecuteScript);
    document.addEventListener("editor-undo", handleUndo);
    document.addEventListener("editor-redo", handleRedo);
    document.addEventListener("editor-paste", handlePaste);

    return () => {
      document.removeEventListener("execute-query", handleExecuteQuery);
      document.removeEventListener("execute-script", handleExecuteScript);
      document.removeEventListener("editor-undo", handleUndo);
      document.removeEventListener("editor-redo", handleRedo);
      document.removeEventListener("editor-paste", handlePaste);
    };
  });

  onDestroy(() => {
    if (editorView) {
      editorView.destroy();
    }
  });

  async function handleConnectionChange(event) {
    const connId = event.target.value;
    selectedConn = $connections.find((c) => c.id === connId);
    activeConnection.set(selectedConn);
    selectedDb = null;
    selectedDatabase.set(null);
    databases = [];
    tables = [];
    schema = {};
    if (selectedConn) {
      await loadDatabases();
    }
  }

  async function handleDatabaseChange(event) {
    selectedDb = event.target.value;
    selectedDatabase.set(selectedDb);
    await loadTablesAndSchema();
  }

  async function runQuery() {
    if (!selectedConn) {
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
      const result = await executeQuery(selectedConn, query);
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
  <!-- Connection and Database Selector Bar -->
  <div
    class="d-flex align-items-center justify-content-between bg-white border-bottom px-3 py-2"
    style="min-height: 45px; gap: 12px;"
  >
    <div class="d-flex align-items-center gap-3 flex-grow-1">
      <!-- Connection Selector -->
      <div class="d-flex align-items-center gap-2" style="min-width: 200px;">
        <i class="fas fa-plug text-primary" style="font-size: 14px;"></i>
        <select
          class="form-select form-select-sm"
          style="max-width: 250px;"
          on:change={handleConnectionChange}
          value={selectedConn?.id || ""}
        >
          <option value="" disabled>Select Connection</option>
          {#each $connections as conn}
            <option value={conn.id}>{conn.name}</option>
          {/each}
        </select>
      </div>

      <!-- Database Selector -->
      {#if selectedConn}
        <div class="d-flex align-items-center gap-2" style="min-width: 200px;">
          <i class="fas fa-database text-success" style="font-size: 14px;"></i>
          {#if loadingDatabases}
            <span
              class="spinner-border spinner-border-sm text-primary"
              role="status"
            >
              <span class="visually-hidden">Loading...</span>
            </span>
          {:else}
            <select
              class="form-select form-select-sm"
              style="max-width: 250px;"
              on:change={handleDatabaseChange}
              value={selectedDb || ""}
              disabled={databases.length === 0}
            >
              <option value="" disabled>Select Database</option>
              {#each databases as db}
                <option value={db.name}>{db.name}</option>
              {/each}
            </select>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Action Buttons -->
    <div class="d-flex align-items-center gap-2">
      <div class="btn-group btn-group-sm">
        <button
          class="btn btn-success d-flex align-items-center gap-1"
          on:click={runQuery}
          disabled={executing || !selectedConn}
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

      {#if !selectedConn}
        <span
          class="badge bg-warning text-dark d-flex align-items-center gap-1"
          style="font-size: 11px; padding: 4px 8px;"
        >
          <i class="fas fa-exclamation-triangle"></i>
          <span>No connection</span>
        </span>
      {:else if !selectedDb}
        <span
          class="badge bg-info text-dark d-flex align-items-center gap-1"
          style="font-size: 11px; padding: 4px 8px;"
        >
          <i class="fas fa-info-circle"></i>
          <span>No database</span>
        </span>
      {/if}
    </div>
  </div>

  <!-- Editor Container -->
  <div bind:this={editorContainer} class="flex-grow-1 editor-wrapper"></div>
</div>

<style>
  .sql-editor-container {
    user-select: text;
    -webkit-user-select: text;
    -moz-user-select: text;
    -ms-user-select: text;
  }

  .editor-wrapper {
    overflow: auto;
    position: relative;
  }

  .sql-editor-container :global(.cm-editor) {
    height: 100%;
  }

  .sql-editor-container :global(.cm-scroller) {
    overflow: auto;
  }

  .sql-editor-container :global(.cm-content),
  .sql-editor-container :global(.cm-line),
  .sql-editor-container :global(.cm-lineNumbers),
  .sql-editor-container :global(.cm-gutters) {
    user-select: text !important;
    -webkit-user-select: text !important;
    -moz-user-select: text !important;
  }

  .sql-editor-container :global(.cm-selectionLayer) {
    z-index: 1;
  }
</style>
