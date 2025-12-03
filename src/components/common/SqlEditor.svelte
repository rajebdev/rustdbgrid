<script>
  import { DatabaseType } from "../../utils/databaseTypes";
  import { onMount, onDestroy } from "svelte";
  import * as monaco from "monaco-editor";
  import { activeConnection, selectedDatabase } from "../../stores/connections";
  import { tabDataStore } from "../../stores/tabData";
  import { activeTheme } from "../../stores/theme";
  import { queryHistoryStore } from "../../stores/queryHistory";
  import { getDefaultQuery } from "../../utils/defaultQueries";
  import { getMonacoTheme } from "../../services/themeService";
  import { formatSql } from "../../utils/sqlFormatter";
  import {
    executeQuery,
    getDatabases,
    getTables,
    getPropertiesObject,
    saveAutoQuery,
    loadAutoQuery,
  } from "../../utils/tauri";
  import { invoke } from "@tauri-apps/api/core";
  import EditorContextMenu from "../context-menus/EditorContextMenu.svelte";

  export let tabId;
  export let tab;

  let editorContainer;
  let editor;
  let contextMenuComponent;
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
  let autoSaveTimeout; // For debouncing auto-save
  let tabData = {}; // Initialize tabData

  // Subscribe to theme changes
  $: if ($activeTheme && $activeTheme !== currentTheme) {
    currentTheme = $activeTheme;
    if (editor) {
      const theme = getMonacoTheme($activeTheme);
      monaco.editor.setTheme(theme);
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
    queryText: getDefaultQuery(selectedConn?.db_type || DatabaseType.MYSQL),
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
          const tableSchema = await getPropertiesObject(
            selectedConn.id,
            "schema",
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
      const tableSchema = await getPropertiesObject(
        selectedConn.id,
        "schema",
        dbName,
        tableName
      );
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

  function createCompletionProvider() {
    return {
      triggerCharacters: [" ", ".", "\n"],
      provideCompletionItems: async (model, position, context, token) => {
        const textUntilPosition = model.getValueInRange({
          startLineNumber: 1,
          startColumn: 1,
          endLineNumber: position.lineNumber,
          endColumn: position.column,
        });

        const line = model.getLineContent(position.lineNumber);
        const textBefore = line.slice(0, position.column - 1);

        // Pre-load tables when user types database name (before dot)
        const dbNameMatch = textBefore.match(/\b(\w+)$/);
        if (dbNameMatch) {
          const typedWord = dbNameMatch[1];
          const matchedDb = databases.find(
            (db) => db.name.toLowerCase() === typedWord.toLowerCase()
          );
          if (matchedDb && !loadedDatabases[matchedDb.name]) {
            loadTablesForDatabase(matchedDb.name).catch((err) =>
              console.error(
                `Background load failed for ${matchedDb.name}:`,
                err
              )
            );
          }
        }

        const suggestions = [];

        // Check for dot notation (table.column or database.table)
        const dotMatch = textBefore.match(/(\w+)\.(\w*)$/);
        if (dotMatch) {
          const [, identifier, partial] = dotMatch;

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
            suggestions.push(
              ...dbTables.map((table) => {
                // If in FROM/JOIN context, add auto-alias
                if (inFromJoinContext) {
                  const alias = generateTableAlias(table.name);
                  return {
                    label: table.name,
                    kind: monaco.languages.CompletionItemKind.Class,
                    insertText: `${table.name} ${alias} `,
                    sortText: "1" + table.name,
                  };
                }
                return {
                  label: table.name,
                  kind: monaco.languages.CompletionItemKind.Class,
                  insertText: table.name,
                  sortText: "1" + table.name,
                };
              })
            );
          } else if (selectedDb && loadedDatabases[selectedDb]?.tables) {
            const matchedTable = loadedDatabases[selectedDb].tables.find(
              (t) => t.name.toLowerCase() === identifier.toLowerCase()
            );
            if (matchedTable) {
              // Load columns for this table
              const columns = await loadColumnsForTable(
                selectedDb,
                matchedTable.name
              );
              suggestions.push(
                ...columns.map((col) => ({
                  label: col,
                  kind: monaco.languages.CompletionItemKind.Field,
                  insertText: col,
                  sortText: "1" + col,
                }))
              );
            }
          }

          // Check if identifier is an alias
          if (suggestions.length === 0 && tableAliasMap.has(identifier)) {
            const tableName = tableAliasMap.get(identifier);
            const dbName = selectedDb;
            if (dbName) {
              const columns = await loadColumnsForTable(dbName, tableName);
              suggestions.push(
                ...columns.map((col) => ({
                  label: col,
                  kind: monaco.languages.CompletionItemKind.Field,
                  insertText: col,
                  sortText: "1" + col,
                }))
              );
            }
          }

          return { suggestions };
        }

        // Check for FROM/JOIN context to show tables
        const fromMatch = textBefore.match(/(?:FROM|JOIN)\s+(\w*)$/i);
        if (fromMatch) {
          // If database is selected, load its tables
          if (selectedDb) {
            const dbTables = await loadTablesForDatabase(selectedDb);
            suggestions.push(
              ...dbTables.map((table) => {
                const alias = generateTableAlias(table.name);
                return {
                  label: table.name,
                  kind: monaco.languages.CompletionItemKind.Class,
                  insertText: `${table.name} ${alias} `,
                  sortText: "2" + table.name,
                };
              })
            );
          }

          // Also show other databases
          suggestions.push(
            ...databases.map((db) => ({
              label: db.name,
              kind: monaco.languages.CompletionItemKind.Module,
              insertText: db.name,
              detail: "database",
              sortText: "3" + db.name,
            }))
          );

          return { suggestions };
        }

        // SQL Keywords
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
        ];

        // Get the word range for completion
        const word = model.getWordUntilPosition(position);

        // Add SQL keywords
        suggestions.push(
          ...sqlKeywords.map((keyword) => ({
            label: keyword,
            kind: monaco.languages.CompletionItemKind.Keyword,
            insertText: keyword + " ",
            sortText: "0" + keyword,
          }))
        );

        // Add tables from current database
        if (selectedDb && loadedDatabases[selectedDb]) {
          const dbTables = loadedDatabases[selectedDb].tables || [];
          suggestions.push(
            ...dbTables.map((table) => ({
              label: table.name,
              kind: monaco.languages.CompletionItemKind.Class,
              insertText: table.name,
              sortText: "4" + table.name,
            }))
          );
        }

        // Add databases
        suggestions.push(
          ...databases.map((db) => ({
            label: db.name,
            kind: monaco.languages.CompletionItemKind.Module,
            insertText: db.name,
            detail: "database",
            sortText: "5" + db.name,
          }))
        );

        return { suggestions };
      },
    };
  }

  function updateEditorCompletions() {
    if (!editor) return;

    // Dispose old completion provider if exists
    if (editor._completionDisposable) {
      editor._completionDisposable.dispose();
    }

    // Register new completion provider
    editor._completionDisposable =
      monaco.languages.registerCompletionItemProvider(
        "sql",
        createCompletionProvider()
      );
  }

  onMount(async () => {
    const theme = getMonacoTheme($activeTheme);

    // Check for auto-saved query from backend
    // Priority: tab.initialContent > tabData.queryText > default query
    let initialQuery =
      tab?.initialContent ||
      tabData.queryText ||
      getDefaultQuery(selectedConn?.db_type || DatabaseType.MYSQL);

    try {
      const autoSaved = await loadAutoQuery();
      if (autoSaved && autoSaved.tab_id === tabId) {
        // Use auto-saved query if available and matches current tab
        initialQuery = autoSaved.query;
      }
    } catch (e) {
      console.error("Failed to load auto-saved query:", e);
    }

    // Create editor
    editor = monaco.editor.create(editorContainer, {
      value: initialQuery,
      language: "sql",
      theme,
      automaticLayout: true,
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      wordWrap: "on",
      fontSize: 13,
      fontFamily: "'Fira Code', 'Monaco', monospace",
      folding: true,
      formatOnPaste: true,
      formatOnType: true,
      contextmenu: false,
      suggest: {
        shareSuggestSelections: true,
        showIcons: true,
      },
    });

    // Register SQL completion provider
    updateEditorCompletions();

    // Listen for changes
    editor.onDidChangeModelContent(() => {
      const text = editor.getValue();
      tabDataStore.setQueryText(tabId, text);
      autoSaveQuery(); // Auto-save with debouncing
    });

    // Add context menu listener
    editorContainer.addEventListener("contextmenu", handleEditorContextMenu);

    // Add keyboard shortcuts
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () =>
      executeSelectedQuery()
    );
    editor.addCommand(
      monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.Enter,
      () => executeSelectedInNewTab()
    );
    editor.addCommand(
      monaco.KeyMod.CtrlCmd | monaco.KeyMod.Alt | monaco.KeyCode.KeyL,
      () => formatSelectedText()
    );
    editor.addCommand(
      monaco.KeyMod.CtrlCmd |
        monaco.KeyMod.Shift |
        monaco.KeyMod.Alt |
        monaco.KeyCode.KeyL,
      () => formatAllText()
    );
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () =>
      saveQuery()
    );

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
        runQuery();
      }
    };

    const handleUndo = (event) => {
      if (event.detail.tabId === tabId && editor) {
        editor.trigger("keyboard", "undo", null);
      }
    };

    const handleRedo = (event) => {
      if (event.detail.tabId === tabId && editor) {
        editor.trigger("keyboard", "redo", null);
      }
    };

    const handlePaste = async (event) => {
      if (event.detail.tabId === tabId && editor && event.detail.text) {
        const selection = editor.getSelection();
        const range = new monaco.Range(
          selection.startLineNumber,
          selection.startColumn,
          selection.endLineNumber,
          selection.endColumn
        );
        editor.executeEdits("paste", [{ range, text: event.detail.text }]);
      }
    };

    const handleLoadQuery = (event) => {
      if (editor && event.detail.query) {
        editor.setValue(event.detail.query);
      }
    };

    document.addEventListener("execute-query", handleExecuteQuery);
    document.addEventListener("execute-script", handleExecuteScript);
    document.addEventListener("editor-undo", handleUndo);
    document.addEventListener("editor-redo", handleRedo);
    document.addEventListener("editor-paste", handlePaste);
    window.addEventListener("load-query", handleLoadQuery);

    return () => {
      editorContainer.removeEventListener(
        "contextmenu",
        handleEditorContextMenu
      );
      document.removeEventListener("execute-query", handleExecuteQuery);
      document.removeEventListener("execute-script", handleExecuteScript);
      document.removeEventListener("editor-undo", handleUndo);
      document.removeEventListener("editor-redo", handleRedo);
      document.removeEventListener("editor-paste", handlePaste);
      window.removeEventListener("load-query", handleLoadQuery);
    };
  });

  onDestroy(() => {
    if (editor) {
      if (editor._completionDisposable) {
        editor._completionDisposable.dispose();
      }
      editor.dispose();
    }
  });

  function addLimitClause(query, dbType) {
    let trimmedQuery = query.trim();
    const upperQuery = trimmedQuery.toUpperCase();

    // Remove trailing semicolon if present
    if (trimmedQuery.endsWith(";")) {
      trimmedQuery = trimmedQuery.slice(0, -1).trim();
    }

    // Only add limit to SELECT queries (check if contains SELECT, not just starts with)
    // This handles cases with comments at the beginning
    if (!upperQuery.includes("SELECT")) {
      return trimmedQuery;
    }

    // Check if query already has a limit/top clause
    if (
      upperQuery.includes("LIMIT") ||
      upperQuery.includes("TOP ") ||
      upperQuery.includes("ROWNUM") ||
      upperQuery.includes("FETCH")
    ) {
      return trimmedQuery;
    }

    // Add appropriate limit clause based on database type
    const dbTypeUpper = (dbType || "").toUpperCase();

    if (dbTypeUpper.includes(DatabaseType.MSSQL.toUpperCase())) {
      // SQL Server: insert TOP before SELECT
      return trimmedQuery.replace(/SELECT\s+/i, "SELECT TOP 200 ");
    } else {
      // MySQL, PostgreSQL, SQLite, etc.: append LIMIT
      return trimmedQuery + " LIMIT 200";
    }
  }

  async function runQuery() {
    if (!selectedConn || !selectedDb) {
      alert("Please select connection and database first");
      return;
    }

    const selection = editor.getSelection();
    const hasSelection = !(
      selection.startLineNumber === selection.endLineNumber &&
      selection.startColumn === selection.endColumn
    );

    let query = "";

    if (hasSelection) {
      // Execute selected text if there is a selection
      query = editor.getModel().getValueInRange(selection);
      if (!query.trim()) {
        alert("No text selected");
        return;
      }
    } else {
      // Execute full query if no selection
      query = editor.getValue();
      if (!query.trim()) {
        alert("Please enter a query");
        return;
      }
    }

    // Use executeQueryText with createNewTab = false
    executeQueryText(query.trim(), false);
  }

  function autoSaveQuery() {
    if (!editor) return;

    // Clear previous timeout
    if (autoSaveTimeout) {
      clearTimeout(autoSaveTimeout);
    }

    // Debounce auto-save: wait 2 seconds after user stops typing
    autoSaveTimeout = setTimeout(async () => {
      const queryText = editor.getValue();
      if (queryText.trim()) {
        try {
          // Save to backend file system (.autosave.json)
          await saveAutoQuery(tabId, queryText, selectedConn?.id, selectedDb);

          // Also save to the actual file if tab has a filePath
          if (tab?.filePath) {
            const configDir = await invoke("get_config_dir");

            // Build full absolute path
            let fullPath = tab.filePath;
            if (
              !fullPath.match(/^[a-zA-Z]:[\\\\/]/) &&
              !fullPath.startsWith("/")
            ) {
              // Relative path, make it absolute
              const sep = navigator.platform.toLowerCase().includes("win")
                ? "\\"
                : "/";
              fullPath =
                configDir +
                sep +
                "rustdbgrid" +
                sep +
                tab.filePath.replace(/\//g, sep);
            }

            await invoke("auto_save_query_file", {
              filePath: fullPath,
              content: queryText,
            });
          }
        } catch (error) {
          console.error("Failed to auto-save query:", error);
        }
      }
    }, 2000);
  }

  function clearEditor() {
    if (editor) {
      editor.setValue("");
      tableAliasMap.clear();
    }
  }

  function handleEditorContextMenu(event) {
    const selection = editor?.getSelection();
    const hasSelection =
      selection &&
      !(
        selection.startLineNumber === selection.endLineNumber &&
        selection.startColumn === selection.endColumn
      );

    const menuItems = [
      {
        id: "execute-selected",
        label: "Execute Selected",
        icon: "fas fa-play",
        shortcut: "Ctrl+Enter",
        onClick: () => executeSelectedQuery(),
        disabled: !hasSelection || !selectedConn || !selectedDb,
      },
      {
        id: "execute-selected-newtab",
        label: "Execute in New Tab",
        icon: "fas fa-arrow-right",
        shortcut: "Ctrl+Shift+Enter",
        onClick: () => executeSelectedInNewTab(),
        disabled: !hasSelection || !selectedConn || !selectedDb,
      },
      {
        id: "separator-1",
        label: "---",
        disabled: true,
      },
      {
        id: "cut",
        label: "Cut",
        icon: "fas fa-cut",
        shortcut: "Ctrl+X",
        onClick: () => {
          if (editor) {
            editor.focus();
            document.execCommand("cut");
          }
        },
        disabled: !hasSelection,
      },
      {
        id: "copy",
        label: "Copy",
        icon: "fas fa-copy",
        shortcut: "Ctrl+C",
        onClick: () => {
          if (editor) {
            editor.focus();
            document.execCommand("copy");
          }
        },
        disabled: !hasSelection,
      },
      {
        id: "paste",
        label: "Paste",
        icon: "fas fa-paste",
        shortcut: "Ctrl+V",
        onClick: async () => {
          if (editor) {
            editor.focus();
            try {
              const text = await navigator.clipboard.readText();
              const selection = editor.getSelection();
              editor.executeEdits("", [
                {
                  range: selection,
                  text: text,
                  forceMoveMarkers: true,
                },
              ]);
            } catch (e) {
              console.error("Paste failed:", e);
            }
          }
        },
      },
      {
        id: "separator-2",
        label: "---",
        disabled: true,
      },
      {
        id: "format-selected",
        label: "Format Selected",
        icon: "fas fa-align-left",
        shortcut: "Ctrl+Alt+L",
        onClick: () => formatSelectedText(),
        disabled: !hasSelection,
      },
      {
        id: "format-all",
        label: "Format All",
        icon: "fas fa-align-justify",
        shortcut: "Ctrl+Shift+Alt+L",
        onClick: () => formatAllText(),
      },
    ];

    contextMenuComponent.show(event, menuItems);
  }

  function executeSelectedQuery() {
    const selection = editor.getSelection();
    const hasSelection = !(
      selection.startLineNumber === selection.endLineNumber &&
      selection.startColumn === selection.endColumn
    );

    if (!selectedConn || !selectedDb) {
      alert("Please select connection and database first");
      return;
    }

    let text = "";

    if (hasSelection) {
      text = editor.getModel().getValueInRange(selection);
      if (!text.trim()) {
        alert("No text selected");
        return;
      }
    } else {
      text = editor.getValue();
      if (!text.trim()) {
        alert("Please enter a query");
        return;
      }
    }

    executeQueryText(text, false);
  }

  function executeSelectedInNewTab() {
    const selection = editor.getSelection();
    const hasSelection = !(
      selection.startLineNumber === selection.endLineNumber &&
      selection.startColumn === selection.endColumn
    );

    if (!selectedConn || !selectedDb) {
      alert("Please select connection and database first");
      return;
    }

    let text = "";

    if (hasSelection) {
      text = editor.getModel().getValueInRange(selection);
      if (!text.trim()) {
        alert("No text selected");
        return;
      }
    } else {
      text = editor.getValue();
      if (!text.trim()) {
        alert("Please enter a query");
        return;
      }
    }

    executeQueryText(text, true);
  }

  async function executeQueryText(query, createNewTab = false) {
    executing = true;

    // Dispatch execution start event
    window.dispatchEvent(
      new CustomEvent("query-execution-start", {
        detail: { tabId },
      })
    );

    try {
      const queryWithLimit = addLimitClause(query, selectedConn.db_type);
      const startTime = Date.now();

      const result = await executeQuery(selectedConn, queryWithLimit);

      const executionTime = Date.now() - startTime;

      // Add to history
      queryHistoryStore.addToHistory(
        query,
        selectedConn.id,
        selectedDb,
        executionTime
      );

      if (createNewTab) {
        // Create new result tab inside QueryTabContent
        window.dispatchEvent(
          new CustomEvent("execute-new-result-tab", {
            detail: {
              tabId,
              result,
              query,
              executionTime,
            },
          })
        );
      } else {
        // Update current result tab
        window.dispatchEvent(
          new CustomEvent("update-result-tab", {
            detail: {
              tabId,
              result,
              query,
              executionTime,
            },
          })
        );
        // Also update store for persistence
        tabDataStore.setQueryResult(tabId, result);
        tabDataStore.setExecutedQuery(tabId, query);
      }
    } catch (error) {
      alert("Query execution failed: " + error);
      console.error(error);
    } finally {
      executing = false;

      // Dispatch execution end event
      window.dispatchEvent(
        new CustomEvent("query-execution-end", {
          detail: { tabId },
        })
      );
    }
  }

  function formatSelectedText() {
    const selection = editor.getSelection();
    const hasSelection = !(
      selection.startLineNumber === selection.endLineNumber &&
      selection.startColumn === selection.endColumn
    );

    if (!hasSelection) {
      // If no selection, format all
      formatAllText();
      return;
    }

    const text = editor.getModel().getValueInRange(selection);
    if (!text.trim()) {
      alert("No text selected");
      return;
    }

    const formatted = formatSql(text, false);

    const range = new monaco.Range(
      selection.startLineNumber,
      selection.startColumn,
      selection.endLineNumber,
      selection.endColumn
    );
    editor.executeEdits("format", [{ range, text: formatted }]);
  }

  function formatAllText() {
    const text = editor.getValue();
    const formatted = formatSql(text, true);
    editor.setValue(formatted);
  }

  function saveQuery() {
    // Dispatch event to trigger save via menuHandlers
    document.dispatchEvent(
      new CustomEvent("save-query", {
        detail: { tabId },
      })
    );
  }
</script>

<div class="sql-editor-container h-100 d-flex flex-column">
  <!-- Editor Area with Action Buttons on Left -->
  <div class="d-flex flex-grow-1 overflow-hidden">
    <!-- Action Buttons Sidebar -->
    <div
      class="editor-actions-sidebar d-flex flex-column align-items-center py-2 gap-1"
    >
      <button
        class="btn-action btn-execute"
        on:click={runQuery}
        disabled={executing || !selectedConn}
        title="Execute (Ctrl+Enter)"
      >
        <i class="fas fa-play"></i>
      </button>
      <button
        class="btn-action btn-execute-new"
        on:click={executeSelectedInNewTab}
        disabled={executing || !selectedConn || !selectedDb}
        title="Execute in New Tab (Ctrl+Shift+Enter)"
      >
        <i class="fas fa-play"></i>
        <i class="fas fa-plus icon-plus"></i>
      </button>

      {#if executing}
        <div class="executing-indicator">
          <i class="fas fa-spinner fa-spin"></i>
        </div>
      {/if}
    </div>

    <!-- Editor Container -->
    <div bind:this={editorContainer} class="flex-grow-1 editor-wrapper"></div>
  </div>
</div>

<!-- Context Menu -->
<EditorContextMenu bind:this={contextMenuComponent} />

<style>
  .sql-editor-container {
    user-select: text;
    -webkit-user-select: text;
    -moz-user-select: text;
    -ms-user-select: text;
  }

  .editor-actions-sidebar {
    background: var(--bg-tertiary);
    border-right: 1px solid var(--border-color);
    padding: 3px;
    width: 24px;
    flex-shrink: 0;
  }

  .btn-action {
    width: 20px;
    height: 20px;
    border: none;
    border-radius: 3px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 9px;
    transition: all 0.15s ease;
    position: relative;
  }

  .btn-action:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-execute {
    background: var(--accent-green, #22c55e);
    color: white;
  }

  .btn-execute:hover:not(:disabled) {
    background: var(--accent-green-hover, #16a34a);
    transform: scale(1.05);
  }

  .btn-execute-new {
    background: var(--accent-blue, #3b82f6);
    color: white;
  }

  .btn-execute-new:hover:not(:disabled) {
    background: var(--accent-blue-hover, #2563eb);
    transform: scale(1.05);
  }

  .btn-execute-new .icon-plus {
    font-size: 5px;
    position: absolute;
    bottom: 1px;
    right: 1px;
  }

  .executing-indicator {
    color: var(--accent-blue);
    font-size: 12px;
    margin-top: 4px;
  }

  .editor-wrapper {
    overflow: auto;
    position: relative;
    background: var(--editor-bg);
  }
</style>
