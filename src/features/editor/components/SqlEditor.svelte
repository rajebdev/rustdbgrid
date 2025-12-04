<script>
  import { onMount } from "svelte";
  import {
    activeConnection,
    selectedDatabase,
  } from "../../connection/stores/connections";
  import { tabDataStore } from "../../../shared/stores/tabData";
  import { tabStore } from "../../../shared/stores/tabs";
  import { activeTheme } from "../../settings/stores/theme";
  import { getDefaultQuery } from "../../../core/constants/defaultQueries";
  import { DatabaseType } from "../../../core/config/databaseTypes";
  import { formatSql } from "../../../shared/utils/sql/sqlFormatter";

  // Services
  import { loadDatabasesForEditor } from "../services/sqlEditorService";
  import { createCompletionProvider } from "../services/editorAutoCompleteService";
  import {
    executeQuery,
    validateQuery,
    cleanQuery,
  } from "../../query/services/queryExecutionService";

  // Components
  import EditorToolbar from "./partials/EditorToolbar.svelte";
  import MonacoEditorView from "./views/MonacoEditorView.svelte";
  import EditorContextMenu from "./EditorContextMenu.svelte";

  export let tabId;
  export let tab;

  // State
  let monacoEditor;
  let contextMenuComponent;
  let executing = false;
  let databases = [];
  let selectedConn = null;
  let selectedDb = null;
  let loadedDatabases = {};
  let tableAliasMap = new Map();

  // Reactive statements
  $: if ($activeConnection) {
    selectedConn = $activeConnection;
    loadDatabases();
  }

  $: if ($selectedDatabase) {
    selectedDb = $selectedDatabase;
  }

  $: tabData = $tabDataStore[tabId] || {
    queryText: getDefaultQuery(selectedConn?.db_type || DatabaseType.MYSQL),
  };

  async function loadDatabases() {
    if (!selectedConn) return;
    try {
      databases = await loadDatabasesForEditor(selectedConn.id);
      if (databases.length > 0 && !selectedDb) {
        selectedDb = databases[0].name;
        selectedDatabase.set(selectedDb);
      }
    } catch (error) {
      console.error("Failed to load databases:", error);
    }
  }

  function createCompletionProviderInstance() {
    return createCompletionProvider({
      selectedConn,
      selectedDb,
      databases,
      loadedDatabases,
      tableAliasMap,
    });
  }

  async function handleEditorReady(editor) {
    monacoEditor = editor;

    // Setup keyboard shortcuts
    const monaco = await import("monaco-editor");
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () =>
      handleExecute()
    );
    editor.addCommand(
      monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.Enter,
      () => handleExecuteNewTab()
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
  }

  function handleEditorChange(text) {
    const currentText = tabDataStore.get(tabId)?.queryText || "";
    const initialContent = tab?.initialContent || "";

    // Update query text
    tabDataStore.setQueryText(tabId, text);

    // Mark as modified if text has changed from initial content
    // Only for query tabs
    if (tab?.type === "query") {
      const hasChanges = text !== initialContent;
      tabStore.markTabAsModified(tabId, hasChanges);
    }
  }

  async function handleExecute() {
    if (executing) return;

    const query = monacoEditor?.getValue() || "";
    const selection = monacoEditor?.getSelection();
    const hasSelection =
      selection &&
      !(
        selection.startLineNumber === selection.endLineNumber &&
        selection.startColumn === selection.endColumn
      );

    let queryText = hasSelection
      ? monacoEditor.getModel().getValueInRange(selection)
      : query;

    const validation = validateQuery(queryText, selectedConn, selectedDb);
    if (!validation.valid) {
      alert(validation.error);
      return;
    }

    try {
      const cleanedQuery = cleanQuery(queryText, selectedConn.db_type);
      await executeQueryInternal(cleanedQuery, false);
    } catch (error) {
      alert(error.message);
    }
  }

  async function handleExecuteNewTab() {
    if (executing) return;

    const query = monacoEditor?.getValue() || "";
    const selection = monacoEditor?.getSelection();
    const hasSelection =
      selection &&
      !(
        selection.startLineNumber === selection.endLineNumber &&
        selection.startColumn === selection.endColumn
      );

    let queryText = hasSelection
      ? monacoEditor.getModel().getValueInRange(selection)
      : query;

    const validation = validateQuery(queryText, selectedConn, selectedDb);
    if (!validation.valid) {
      alert(validation.error);
      return;
    }

    try {
      const cleanedQuery = cleanQuery(queryText, selectedConn.db_type);
      await executeQueryInternal(cleanedQuery, true);
    } catch (error) {
      alert(error.message);
    }
  }

  async function executeQueryInternal(query, createNewTab) {
    executing = true;

    window.dispatchEvent(
      new CustomEvent("query-execution-start", { detail: { tabId } })
    );

    try {
      const {
        result,
        query: cleanedQuery,
        executionTime,
      } = await executeQuery({
        query,
        connId: selectedConn.id,
        dbType: selectedConn.db_type,
        dbName: selectedDb,
        tabId,
        createNewTab,
      });

      if (createNewTab) {
        window.dispatchEvent(
          new CustomEvent("execute-new-result-tab", {
            detail: { tabId, result, query: cleanedQuery, executionTime },
          })
        );
      } else {
        window.dispatchEvent(
          new CustomEvent("update-result-tab", {
            detail: { tabId, result, query: cleanedQuery, executionTime },
          })
        );
        tabDataStore.setQueryResult(tabId, result);
        tabDataStore.setExecutedQuery(tabId, cleanedQuery);
      }
    } catch (error) {
      alert("Query execution failed: " + error);
      console.error(error);
    } finally {
      executing = false;
      window.dispatchEvent(
        new CustomEvent("query-execution-end", { detail: { tabId } })
      );
    }
  }

  function formatSelectedText() {
    if (!monacoEditor) return;

    const selection = monacoEditor.getSelection();
    const hasSelection = !(
      selection.startLineNumber === selection.endLineNumber &&
      selection.startColumn === selection.endColumn
    );

    if (!hasSelection) {
      formatAllText();
      return;
    }

    const text = monacoEditor.getModel().getValueInRange(selection);
    if (!text.trim()) {
      alert("No text selected");
      return;
    }

    const monaco = monacoEditor.constructor;
    const formatted = formatSql(text);
    const range = new monaco.Range(
      selection.startLineNumber,
      selection.startColumn,
      selection.endLineNumber,
      selection.endColumn
    );
    monacoEditor.executeEdits("format", [{ range, text: formatted }]);
  }

  function formatAllText() {
    if (!monacoEditor) return;
    const text = monacoEditor.getValue();
    const formatted = formatSql(text);
    monacoEditor.setValue(formatted);
  }

  function saveQuery() {
    document.dispatchEvent(
      new CustomEvent("save-query", { detail: { tabId } })
    );
  }

  function handleEditorContextMenu(event) {
    const selection = monacoEditor?.getSelection();
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
        onClick: () => handleExecute(),
        disabled: !hasSelection || !selectedConn || !selectedDb,
      },
      {
        id: "execute-selected-newtab",
        label: "Execute in New Tab",
        icon: "fas fa-arrow-right",
        shortcut: "Ctrl+Shift+Enter",
        onClick: () => handleExecuteNewTab(),
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
          if (monacoEditor) {
            monacoEditor.focus();
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
          if (monacoEditor) {
            monacoEditor.focus();
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
          if (monacoEditor) {
            monacoEditor.focus();
            try {
              const text = await navigator.clipboard.readText();
              const selection = monacoEditor.getSelection();
              monacoEditor.executeEdits("", [
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

  onMount(async () => {
    // Load initial query
    let initialQuery =
      tab?.initialContent ||
      tabData.queryText ||
      getDefaultQuery(selectedConn?.db_type || DatabaseType.MYSQL);

    // Load databases if connection is selected
    if (selectedConn) {
      loadDatabases();
    }

    // Setup event listeners
    const handleExecuteQuery = (event) => {
      if (event.detail.tabId === tabId) {
        handleExecute();
      }
    };

    const handleExecuteScript = (event) => {
      if (event.detail.tabId === tabId) {
        handleExecute();
      }
    };

    const handleUndo = (event) => {
      if (event.detail.tabId === tabId && monacoEditor) {
        monacoEditor.trigger("keyboard", "undo", null);
      }
    };

    const handleRedo = (event) => {
      if (event.detail.tabId === tabId && monacoEditor) {
        monacoEditor.trigger("keyboard", "redo", null);
      }
    };

    const handlePaste = async (event) => {
      if (event.detail.tabId === tabId && monacoEditor && event.detail.text) {
        const monaco = await import("monaco-editor");
        const selection = monacoEditor.getSelection();
        const range = new monaco.Range(
          selection.startLineNumber,
          selection.startColumn,
          selection.endLineNumber,
          selection.endColumn
        );
        monacoEditor.executeEdits("paste", [
          { range, text: event.detail.text },
        ]);
      }
    };

    const handleLoadQuery = (event) => {
      if (monacoEditor && event.detail.query) {
        monacoEditor.setValue(event.detail.query);
      }
    };

    document.addEventListener("execute-query", handleExecuteQuery);
    document.addEventListener("execute-script", handleExecuteScript);
    document.addEventListener("editor-undo", handleUndo);
    document.addEventListener("editor-redo", handleRedo);
    document.addEventListener("editor-paste", handlePaste);
    window.addEventListener("load-query", handleLoadQuery);

    return () => {
      document.removeEventListener("execute-query", handleExecuteQuery);
      document.removeEventListener("execute-script", handleExecuteScript);
      document.removeEventListener("editor-undo", handleUndo);
      document.removeEventListener("editor-redo", handleRedo);
      document.removeEventListener("editor-paste", handlePaste);
      window.removeEventListener("load-query", handleLoadQuery);
    };
  });
</script>

<div class="sql-editor-container h-100 d-flex flex-column">
  <div class="d-flex flex-grow-1 overflow-hidden">
    <EditorToolbar
      {executing}
      {selectedConn}
      {selectedDb}
      onExecute={handleExecute}
      onExecuteNewTab={handleExecuteNewTab}
    />

    <MonacoEditorView
      value={tabData.queryText}
      theme={$activeTheme}
      completionProvider={createCompletionProviderInstance()}
      onChange={handleEditorChange}
      onContextMenu={handleEditorContextMenu}
      onReady={handleEditorReady}
    />
  </div>
</div>

<EditorContextMenu bind:this={contextMenuComponent} />

<style>
  .sql-editor-container {
    user-select: text;
    -webkit-user-select: text;
    -moz-user-select: text;
    -ms-user-select: text;
  }
</style>
