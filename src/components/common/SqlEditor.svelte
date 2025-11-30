<script>
  import { onMount, onDestroy } from "svelte";
  import { EditorView, basicSetup } from "codemirror";
  import { EditorState } from "@codemirror/state";
  import { drawSelection, highlightActiveLineGutter } from "@codemirror/view";
  import { sql } from "@codemirror/lang-sql";
  import { autocompletion } from "@codemirror/autocomplete";
  import { oneDark } from "@codemirror/theme-one-dark";
  import {
    activeConnection,
    connections,
    selectedDatabase,
  } from "../../stores/connections";
  import { tabDataStore } from "../../stores/tabData";
  import { activeTheme } from "../../stores/theme";
  import { getDefaultQuery } from "../../utils/defaultQueries";
  import { getEditorTheme } from "../../services/themeService";
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
  let currentTheme = null;
  let loadedDatabases = {}; // Cache for database tables
  let tableAliasMap = new Map(); // Track used aliases to prevent duplicates

  // Subscribe to theme changes
  $: if ($activeTheme && $activeTheme !== currentTheme) {
    currentTheme = $activeTheme;
    if (editorView) {
      updateEditorExtensions();
    }
  }

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

      // Cache tables for selected database
      if (selectedDb) {
        loadedDatabases[selectedDb] = { tables, schema: newSchema };
      }

      // Update editor with new schema
      if (editorView) {
        updateEditorExtensions();
      }
    } catch (error) {
      console.error("Failed to load tables:", error);
    }
  }

  // Helper to generate table alias
  function generateTableAlias(tableName) {
    // Convert to lowercase and get first letter of each word
    const words = tableName.toLowerCase().split(/[_-]/);
    let alias = "";

    if (words.length === 1) {
      // Single word: take first letter
      alias = words[0][0];
    } else {
      // Multiple words: take first letter of each
      alias = words.map((w) => w[0]).join("");
    }

    // Check for duplicates and append number if needed
    let finalAlias = alias;
    let counter = 1;
    const existingAliases = new Set(tableAliasMap.values());

    while (existingAliases.has(finalAlias)) {
      finalAlias = alias + counter;
      counter++;
    }

    return finalAlias;
  }

  // Lazy load tables for a specific database
  async function loadTablesForDatabase(dbName) {
    if (!selectedConn || !dbName) return [];

    // Return from cache if available
    if (loadedDatabases[dbName]) {
      return loadedDatabases[dbName].tables;
    }

    try {
      const dbTables = await getTables(selectedConn, dbName);

      // Only store table names, don't load schema yet
      loadedDatabases[dbName] = { tables: dbTables, schema: {} };
      return dbTables;
    } catch (error) {
      console.error(`Failed to load tables for ${dbName}:`, error);
      return [];
    }
  }

  // Lazy load columns for a specific table
  async function loadColumnsForTable(dbName, tableName) {
    if (!selectedConn || !dbName || !tableName) return [];

    // Check if already loaded
    if (loadedDatabases[dbName]?.schema?.[tableName]) {
      return loadedDatabases[dbName].schema[tableName];
    }

    try {
      const tableSchema = await getTableSchema(selectedConn, dbName, tableName);
      const columns = tableSchema.columns.map((col) => col.name);

      if (!loadedDatabases[dbName]) {
        loadedDatabases[dbName] = { tables: [], schema: {} };
      }
      if (!loadedDatabases[dbName].schema) {
        loadedDatabases[dbName].schema = {};
      }
      loadedDatabases[dbName].schema[tableName] = columns;

      return columns;
    } catch (error) {
      console.error(
        `Failed to load columns for ${dbName}.${tableName}:`,
        error
      );
      return [];
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
      apply: (view, completion, from, to) => {
        // Insert keyword with space
        const text = `${keyword} `;
        view.dispatch({
          changes: { from, to, insert: text },
          selection: { anchor: from + text.length },
        });
      },
    }));

    return async function sqlAutoComplete(context) {
      const line = context.state.doc.lineAt(context.pos);
      const textBefore = line.text.slice(0, context.pos - line.from);

      // Pre-load tables when user types database name (before dot)
      const dbNameMatch = textBefore.match(/\b(\w+)$/);
      if (dbNameMatch) {
        const typedWord = dbNameMatch[1];
        const matchedDb = databases.find(
          (db) => db.name.toLowerCase() === typedWord.toLowerCase()
        );
        if (matchedDb && !loadedDatabases[matchedDb.name]) {
          // Pre-load tables in background without blocking
          loadTablesForDatabase(matchedDb.name).catch((err) =>
            console.error(`Background load failed for ${matchedDb.name}:`, err)
          );
        }
      }

      // Check for dot notation (table.column or database.table)
      const dotMatch = textBefore.match(/(\w+)\.(\w*)$/);
      if (dotMatch) {
        const [, identifier, partial] = dotMatch;
        const options = [];

        // Check if we're in FROM/JOIN context for auto-alias
        const beforeDot = textBefore.substring(
          0,
          textBefore.lastIndexOf(identifier)
        );
        const inFromJoinContext = /(?:FROM|JOIN)\s+$/i.test(beforeDot);

        // Check if identifier is a database name (case-insensitive)
        const matchedDb = databases.find(
          (db) => db.name.toLowerCase() === identifier.toLowerCase()
        );

        if (matchedDb) {
          // Tables should already be loaded, just retrieve from cache
          const dbTables = await loadTablesForDatabase(matchedDb.name);
          options.push(
            ...dbTables.map((table) => {
              // If in FROM/JOIN context, add auto-alias
              if (inFromJoinContext) {
                const alias = generateTableAlias(table.name);
                return {
                  label: table.name,
                  type: "type",
                  boost: 10,
                  apply: (view, completion, from, to) => {
                    // Store the alias mapping
                    tableAliasMap.set(alias, table.name);

                    // Insert table name with alias and space
                    const text = `${table.name} ${alias} `;
                    view.dispatch({
                      changes: { from, to, insert: text },
                      selection: { anchor: from + text.length },
                    });
                  },
                };
              }
              // Otherwise, just table name
              return {
                label: table.name,
                type: "type",
                boost: 10,
                apply: table.name,
              };
            })
          );
        }
        // Check if identifier is a table name in current database
        else if (selectedDb && loadedDatabases[selectedDb]?.tables) {
          const matchedTable = loadedDatabases[selectedDb].tables.find(
            (t) => t.name.toLowerCase() === identifier.toLowerCase()
          );
          if (matchedTable) {
            // Load columns for this table
            const columns = await loadColumnsForTable(
              selectedDb,
              matchedTable.name
            );
            options.push(
              ...columns.map((col) => ({
                label: col,
                type: "property",
                boost: 10,
                apply: col,
              }))
            );
          }
        }
        // Check if identifier is an alias
        if (options.length === 0 && tableAliasMap.has(identifier)) {
          const tableName = tableAliasMap.get(identifier);
          const dbName = selectedDb; // Assume current database for alias
          if (dbName) {
            const columns = await loadColumnsForTable(dbName, tableName);
            options.push(
              ...columns.map((col) => ({
                label: col,
                type: "property",
                boost: 10,
                apply: col,
              }))
            );
          }
        }

        if (options.length > 0) {
          return {
            from: context.pos - partial.length,
            options,
            validFor: /^\w*$/,
          };
        }
        return null;
      }

      // Check for FROM/JOIN context to show tables
      const fromMatch = textBefore.match(/(?:FROM|JOIN)\s+(\w*)$/i);
      if (fromMatch) {
        const options = [];

        // If database is selected, load its tables
        if (selectedDb) {
          const dbTables = await loadTablesForDatabase(selectedDb);
          options.push(
            ...dbTables.map((table) => {
              const alias = generateTableAlias(table.name);
              return {
                label: table.name,
                type: "type",
                boost: 10,
                apply: (view, completion, from, to) => {
                  // Store the alias mapping
                  tableAliasMap.set(alias, table.name);

                  // Insert table name with alias and space
                  const text = `${table.name} ${alias} `;
                  view.dispatch({
                    changes: { from, to, insert: text },
                    selection: { anchor: from + text.length },
                  });
                },
              };
            })
          );
        }

        // Also show other databases with pre-load
        options.push(
          ...databases.map((db) => ({
            label: db.name,
            type: "namespace",
            boost: 5,
            detail: "database",
            apply: (view, completion, from, to) => {
              // Pre-load tables when database is selected
              if (!loadedDatabases[db.name]) {
                loadTablesForDatabase(db.name).catch((err) =>
                  console.error(
                    `Failed to pre-load tables for ${db.name}:`,
                    err
                  )
                );
              }
              // Insert database name
              view.dispatch({
                changes: { from, to, insert: db.name },
                selection: { anchor: from + db.name.length },
              });
            },
          }))
        );

        if (options.length > 0) {
          return {
            from: context.pos - fromMatch[1].length,
            options,
            validFor: /^\w*$/,
          };
        }
      }

      // Default word-based completion
      const word = context.matchBefore(/\w*/);
      if (!word || (word.from === word.to && !context.explicit)) {
        return null;
      }

      // Check if we're in FROM/JOIN context for tables with alias
      const lineTextBefore = context.state.doc
        .lineAt(word.from)
        .text.slice(0, word.from - context.state.doc.lineAt(word.from).from);
      const inFromJoinContext = /(?:FROM|JOIN)\s+\w*$/i.test(lineTextBefore);

      // Build basic completions
      const options = [...sqlKeywords];

      // Add tables from current database
      if (selectedDb && loadedDatabases[selectedDb]) {
        const dbTables = loadedDatabases[selectedDb].tables || [];
        options.push(
          ...dbTables.map((table) => {
            // If in FROM/JOIN context, add auto-alias
            if (inFromJoinContext) {
              const alias = generateTableAlias(table.name);
              return {
                label: table.name,
                type: "type",
                boost: 2,
                apply: (view, completion, from, to) => {
                  // Store the alias mapping
                  tableAliasMap.set(alias, table.name);

                  // Insert table name with alias and space
                  const text = `${table.name} ${alias} `;
                  view.dispatch({
                    changes: { from, to, insert: text },
                    selection: { anchor: from + text.length },
                  });
                },
              };
            }
            // Otherwise, just table name
            return {
              label: table.name,
              type: "type",
              boost: 2,
              apply: table.name,
            };
          })
        );
      }

      // Add databases with pre-load on hover/selection
      options.push(
        ...databases.map((db) => ({
          label: db.name,
          type: "namespace",
          boost: 1,
          detail: "database",
          apply: (view, completion, from, to) => {
            // Pre-load tables when database is selected
            if (!loadedDatabases[db.name]) {
              loadTablesForDatabase(db.name).catch((err) =>
                console.error(`Failed to pre-load tables for ${db.name}:`, err)
              );
            }
            // Insert database name
            view.dispatch({
              changes: { from, to, insert: db.name },
              selection: { anchor: from + db.name.length },
            });
          },
        }))
      );

      return {
        from: word.from,
        options,
        validFor: /^\w*$/,
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
        activateOnTyping: true,
      }),
      EditorView.lineWrapping,
    ];

    // Add theme based on current theme
    if ($activeTheme === "dark") {
      extensions.push(oneDark);
    } else {
      extensions.push(EditorView.theme(getEditorTheme($activeTheme)));
    }

    // Add update listener
    extensions.push(
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          const text = update.state.doc.toString();
          tabDataStore.setQueryText(tabId, text);
        }
      })
    );

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
    const extensions = [
      basicSetup,
      drawSelection(),
      highlightActiveLineGutter(),
      sql(),
      autocompletion({
        override: [createAutocompletions()],
        activateOnTyping: true,
      }),
      EditorView.lineWrapping,
    ];

    // Add theme based on current theme
    if ($activeTheme === "dark") {
      extensions.push(oneDark);
    } else {
      extensions.push(EditorView.theme(getEditorTheme($activeTheme)));
    }

    // Add update listener
    extensions.push(
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          const text = update.state.doc.toString();
          tabDataStore.setQueryText(tabId, text);
        }
      })
    );

    editorView = new EditorView({
      doc:
        tabData.queryText || getDefaultQuery(selectedConn?.db_type || "MySQL"),
      extensions,
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
    loadedDatabases = {};
    tableAliasMap.clear();
    if (selectedConn) {
      await loadDatabases();
    }
  }

  async function handleDatabaseChange(event) {
    selectedDb = event.target.value;
    selectedDatabase.set(selectedDb);
    tableAliasMap.clear(); // Reset alias map when database changes
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
      tableAliasMap.clear(); // Reset alias map when clearing editor
    }
  }
</script>

<div class="sql-editor-container h-100 d-flex flex-column">
  <!-- Connection and Database Selector Bar -->
  <div
    class="editor-toolbar d-flex align-items-center justify-content-between border-bottom px-3 py-2"
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

  .editor-toolbar {
    background: var(--bg-secondary);
  }

  .editor-wrapper {
    overflow: auto;
    position: relative;
    background: var(--editor-bg);
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
