<script>
  import { onMount } from "svelte";
  import MainLayout from "./components/layout/MainLayout.svelte";
  import SidebarWrapper from "./components/layout/wrappers/SidebarWrapper.svelte";
  import ContentArea from "./components/layout/wrappers/ContentArea.svelte";
  import ConnectionModal from "./components/modals/ConnectionModal.svelte";
  import SplashScreen from "./components/screens/SplashScreen.svelte";
  import AboutModal from "./components/modals/AboutModal.svelte";
  import KeyboardShortcutsModal from "./components/modals/KeyboardShortcutsModal.svelte";
  import { activeConnection } from "./stores/connections";
  import { tabDataStore } from "./stores/tabData";
  import { getTableData } from "./utils/tauri";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { listen } from "@tauri-apps/api/event";
  import { message } from "@tauri-apps/plugin-dialog";

  // Helper function for showing messages
  async function showMessage(msg, title = "RustDBGrid") {
    try {
      await message(msg, { title, kind: "info" });
    } catch (error) {
      console.error("Dialog error:", error);
      alert(msg); // Fallback to alert if dialog fails
    }
  }

  async function showError(msg, title = "Error") {
    try {
      await message(msg, { title, kind: "error" });
    } catch (error) {
      console.error("Dialog error:", error);
      alert(msg); // Fallback to alert if dialog fails
    }
  }

  let showSidebar = true;
  let showModal = false;
  let showAboutModal = false;
  let showKeyboardShortcutsModal = false;
  let showSplash = true;
  let loadingProgress = 0;
  let loadingMessage = "Initializing application";

  let tabs = [];
  let activeTab = null;

  // Sidebar resize functionality
  let sidebarWidth = 320;
  let minSidebarWidth = 200;
  let maxSidebarWidth = 600;
  let isResizing = false;
  let isWindowMaximized = false;
  let normalSidebarWidth = 320; // Store normal width when not maximized

  // SQL Editor resize functionality
  let editorHeight = 300;
  let minEditorHeight = 150;
  let maxEditorHeight = 700;
  let isResizingEditor = false;

  $: currentTabData = activeTab ? $tabDataStore[activeTab.id] : null;

  onMount(async () => {
    // Listen for window maximize/unmaximize events
    const appWindow = getCurrentWindow();

    // Check initial state
    isWindowMaximized = await appWindow.isMaximized();
    if (isWindowMaximized) {
      normalSidebarWidth = sidebarWidth;
      sidebarWidth = 320;
    }

    // Listen for window resize events
    const unlisten = await listen("tauri://resize", async () => {
      const maximized = await appWindow.isMaximized();

      if (maximized !== isWindowMaximized) {
        isWindowMaximized = maximized;

        if (maximized) {
          // Store current width before maximizing
          normalSidebarWidth = sidebarWidth;
          sidebarWidth = 320;
        } else {
          // Restore previous width when unmaximizing
          sidebarWidth = normalSidebarWidth;
        }
      }
    });

    // Setup keyboard shortcuts
    setupKeyboardShortcuts();

    // Listen for editor resize events from QueryTabContent
    const handleStartEditorResize = (e) => {
      isResizingEditor = true;
      e.detail.event.preventDefault();
    };

    const handleEditorResizeMove = (e) => {
      if (!isResizingEditor) return;
      const event = e.detail.event;
      const mainContent = document.querySelector(".main-content-area");
      if (mainContent) {
        const rect = mainContent.getBoundingClientRect();
        const newHeight = event.clientY - rect.top;
        if (newHeight >= minEditorHeight && newHeight <= maxEditorHeight) {
          editorHeight = newHeight;
        }
      }
    };

    window.addEventListener("start-editor-resize", handleStartEditorResize);
    window.addEventListener("editor-resize-move", handleEditorResizeMove);

    // Simulate initialization steps
    const steps = [
      { message: "Loading configuration", duration: 300 },
      { message: "Initializing database drivers", duration: 400 },
      { message: "Loading saved connections", duration: 300 },
      { message: "Preparing workspace", duration: 200 },
      { message: "Ready", duration: 100 },
    ];

    let currentProgress = 0;
    const progressStep = 100 / steps.length;

    for (let i = 0; i < steps.length; i++) {
      loadingMessage = steps[i].message;
      await new Promise((resolve) => setTimeout(resolve, steps[i].duration));
      currentProgress += progressStep;
      loadingProgress = currentProgress;
    }

    // Hide splash screen with fade out
    await new Promise((resolve) => setTimeout(resolve, 300));
    showSplash = false;

    // Cleanup listener on component destroy
    return () => {
      unlisten();
      window.removeEventListener(
        "start-editor-resize",
        handleStartEditorResize
      );
      window.removeEventListener("editor-resize-move", handleEditorResizeMove);
    };
  });

  function setupKeyboardShortcuts() {
    const handleKeyDown = (event) => {
      // Ctrl/Cmd + N: New Query
      if (
        (event.ctrlKey || event.metaKey) &&
        event.key === "n" &&
        !event.shiftKey
      ) {
        event.preventDefault();
        addNewQueryTab();
      }
      // Ctrl/Cmd + O: Open File
      else if ((event.ctrlKey || event.metaKey) && event.key === "o") {
        event.preventDefault();
        handleOpenFile();
      }
      // Ctrl/Cmd + S: Save Query
      else if (
        (event.ctrlKey || event.metaKey) &&
        event.key === "s" &&
        !event.shiftKey
      ) {
        event.preventDefault();
        handleSaveQuery();
      }
      // Ctrl/Cmd + Shift + S: Save As
      else if (
        (event.ctrlKey || event.metaKey) &&
        event.shiftKey &&
        event.key === "S"
      ) {
        event.preventDefault();
        handleSaveAs();
      }
      // Ctrl/Cmd + B: Toggle Sidebar
      else if ((event.ctrlKey || event.metaKey) && event.key === "b") {
        event.preventDefault();
        showSidebar = !showSidebar;
      }
      // Ctrl/Cmd + Shift + C: New Connection
      else if (
        (event.ctrlKey || event.metaKey) &&
        event.shiftKey &&
        event.key === "C"
      ) {
        event.preventDefault();
        showModal = true;
      }
      // F5 or Ctrl/Cmd + Enter: Execute Query
      else if (
        event.key === "F5" ||
        ((event.ctrlKey || event.metaKey) && event.key === "Enter")
      ) {
        event.preventDefault();
        handleExecute();
      }
      // Ctrl/Cmd + Shift + Enter: Execute Script
      else if (
        (event.ctrlKey || event.metaKey) &&
        event.shiftKey &&
        event.key === "Enter"
      ) {
        event.preventDefault();
        handleExecuteScript();
      }
      // F5 with Shift: Refresh
      else if (event.shiftKey && event.key === "F5") {
        event.preventDefault();
        handleRefresh();
      }
      // Ctrl/Cmd + W: Close Tab
      else if ((event.ctrlKey || event.metaKey) && event.key === "w") {
        event.preventDefault();
        if (activeTab && tabs.length > 0) {
          handleTabClose({ detail: activeTab });
        }
      }
      // Ctrl/Cmd + Tab: Next Tab
      else if (
        (event.ctrlKey || event.metaKey) &&
        event.key === "Tab" &&
        !event.shiftKey
      ) {
        event.preventDefault();
        if (tabs.length > 1) {
          const currentIndex = tabs.findIndex((t) => t.id === activeTab?.id);
          const nextIndex = (currentIndex + 1) % tabs.length;
          activeTab = tabs[nextIndex];
        }
      }
      // Ctrl/Cmd + Shift + Tab: Previous Tab
      else if (
        (event.ctrlKey || event.metaKey) &&
        event.shiftKey &&
        event.key === "Tab"
      ) {
        event.preventDefault();
        if (tabs.length > 1) {
          const currentIndex = tabs.findIndex((t) => t.id === activeTab?.id);
          const prevIndex = (currentIndex - 1 + tabs.length) % tabs.length;
          activeTab = tabs[prevIndex];
        }
      }
      // Ctrl/Cmd + K then Ctrl/Cmd + S: Keyboard Shortcuts
      else if ((event.ctrlKey || event.metaKey) && event.key === "k") {
        event.preventDefault();
        const waitForSecondKey = (e) => {
          if ((e.ctrlKey || e.metaKey) && e.key === "s") {
            e.preventDefault();
            showKeyboardShortcutsModal = true;
            window.removeEventListener("keydown", waitForSecondKey);
          }
          setTimeout(() => {
            window.removeEventListener("keydown", waitForSecondKey);
          }, 1000);
        };
        window.addEventListener("keydown", waitForSecondKey);
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    // Cleanup on destroy
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }

  function handleMenuAction(event) {
    const action = event.type;
    console.log("Menu action:", action);

    switch (action) {
      // File Menu
      case "newQuery":
        addNewQueryTab();
        break;
      case "openFile":
        handleOpenFile();
        break;
      case "saveQuery":
        handleSaveQuery();
        break;
      case "saveAs":
        handleSaveAs();
        break;
      case "export":
        handleExportData();
        break;
      case "import":
        handleImportData();
        break;

      // Edit Menu
      case "undo":
        handleUndo();
        break;
      case "redo":
        handleRedo();
        break;
      case "copy":
        handleCopy();
        break;
      case "paste":
        handlePaste();
        break;

      // View Menu
      case "toggleSidebar":
        showSidebar = !showSidebar;
        break;
      case "toggleToolbar":
        handleToggleToolbar();
        break;
      case "viewColumns":
        handleViewColumns();
        break;

      // Database Menu
      case "newConnection":
        showModal = true;
        break;
      case "connect":
        handleConnect();
        break;
      case "disconnect":
        handleDisconnect();
        break;

      // Toolbar Actions
      case "execute":
        handleExecute();
        break;
      case "executeScript":
        handleExecuteScript();
        break;
      case "stop":
        handleStop();
        break;
      case "commit":
        handleCommit();
        break;
      case "rollback":
        handleRollback();
        break;
      case "refresh":
        handleRefresh();
        break;

      // Help Menu
      case "documentation":
        handleDocumentation();
        break;
      case "about":
        handleAbout();
        break;

      default:
        console.warn("Unknown action:", action);
    }
  }

  // File Menu Handlers
  async function handleOpenFile() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const file = await open({
        title: "Open SQL File",
        filters: [
          {
            name: "SQL Files",
            extensions: ["sql", "txt"],
          },
        ],
      });

      if (file) {
        const { readTextFile } = await import("@tauri-apps/plugin-fs");
        const content = await readTextFile(file.path);

        const newTab = {
          id: Date.now(),
          title: file.name || `Query ${tabs.length + 1}`,
          type: "query",
          modified: false,
          filePath: file.path,
        };
        tabs = [...tabs, newTab];
        activeTab = newTab;

        // Set query text
        tabDataStore.setQueryText(newTab.id, content);
      }
    } catch (error) {
      console.error("Failed to open file:", error);
      await showError("Failed to open file: " + error.message);
    }
  }

  async function handleSaveQuery() {
    if (!activeTab || activeTab.type !== "query") {
      await showError("No query tab is active");
      return;
    }

    const tabData = $tabDataStore[activeTab.id];
    if (!tabData || !tabData.queryText) {
      await showError("No query to save");
      return;
    }

    try {
      if (activeTab.filePath) {
        // Save to existing file
        const { writeTextFile } = await import("@tauri-apps/plugin-fs");
        await writeTextFile(activeTab.filePath, tabData.queryText);

        // Update tab
        activeTab.modified = false;
        tabs = [...tabs];

        await showMessage("Query saved successfully");
      } else {
        // No file path, use Save As
        await handleSaveAs();
      }
    } catch (error) {
      console.error("Failed to save query:", error);
      await showError("Failed to save query: " + error.message);
    }
  }

  async function handleSaveAs() {
    if (!activeTab || activeTab.type !== "query") {
      await showError("No query tab is active");
      return;
    }

    const tabData = $tabDataStore[activeTab.id];
    if (!tabData || !tabData.queryText) {
      await showError("No query to save");
      return;
    }

    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const file = await save({
        title: "Save SQL File",
        filters: [
          {
            name: "SQL Files",
            extensions: ["sql"],
          },
        ],
        defaultPath: `${activeTab.title}.sql`,
      });

      if (file) {
        const { writeTextFile } = await import("@tauri-apps/plugin-fs");
        await writeTextFile(file, tabData.queryText);

        // Update tab
        activeTab.filePath = file;
        activeTab.modified = false;
        const fileName = file.split(/[\\/]/).pop().replace(".sql", "");
        activeTab.title = fileName;
        tabs = [...tabs];

        await showMessage("Query saved successfully");
      }
    } catch (error) {
      console.error("Failed to save query:", error);
      await showError("Failed to save query: " + error.message);
    }
  }

  async function handleExportData() {
    if (!activeTab) {
      await showError("No active tab");
      return;
    }

    const tabData = $tabDataStore[activeTab.id];
    if (!tabData || !tabData.queryResult) {
      await showError("No data to export");
      return;
    }

    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const file = await save({
        title: "Export Data",
        filters: [
          { name: "CSV Files", extensions: ["csv"] },
          { name: "JSON Files", extensions: ["json"] },
          { name: "SQL Insert", extensions: ["sql"] },
        ],
      });

      if (file) {
        const { invoke } = await import("@tauri-apps/api/core");

        // Determine export format from extension
        const ext = file.split(".").pop().toLowerCase();
        await invoke("export_data", {
          data: tabData.queryResult,
          filePath: file,
          format: ext,
        });

        await showMessage("Data exported successfully");
      }
    } catch (error) {
      console.error("Failed to export data:", error);
      await showError("Failed to export data: " + error.message);
    }
  }

  async function handleImportData() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const file = await open({
        title: "Import Data",
        filters: [
          { name: "CSV Files", extensions: ["csv"] },
          { name: "JSON Files", extensions: ["json"] },
          { name: "SQL Files", extensions: ["sql"] },
        ],
      });

      if (file) {
        await showMessage("Import functionality will be available soon");
        // TODO: Implement import logic
      }
    } catch (error) {
      console.error("Failed to import data:", error);
      await showError("Failed to import data: " + error.message);
    }
  }

  // Edit Menu Handlers
  function handleUndo() {
    // Trigger undo in the active editor (if any)
    if (activeTab && activeTab.type === "query") {
      document.dispatchEvent(
        new CustomEvent("editor-undo", { detail: { tabId: activeTab.id } })
      );
    }
  }

  function handleRedo() {
    // Trigger redo in the active editor (if any)
    if (activeTab && activeTab.type === "query") {
      document.dispatchEvent(
        new CustomEvent("editor-redo", { detail: { tabId: activeTab.id } })
      );
    }
  }

  async function handleCopy() {
    // Use clipboard API
    try {
      const selectedText = window.getSelection().toString();
      if (selectedText) {
        await navigator.clipboard.writeText(selectedText);
      }
    } catch (error) {
      console.error("Failed to copy:", error);
    }
  }

  async function handlePaste() {
    // Trigger paste in the active editor
    if (activeTab && activeTab.type === "query") {
      try {
        const text = await navigator.clipboard.readText();
        document.dispatchEvent(
          new CustomEvent("editor-paste", {
            detail: { tabId: activeTab.id, text },
          })
        );
      } catch (error) {
        console.error("Failed to paste:", error);
      }
    }
  }

  // View Menu Handlers
  let showToolbar = true;

  // Query execution tracking
  let runningQueries = new Map(); // Map<tabId, AbortController>

  function handleToggleToolbar() {
    showToolbar = !showToolbar;
  }

  async function handleViewColumns() {
    if (!activeTab) {
      await showError("No active tab");
      return;
    }

    const tabData = $tabDataStore[activeTab.id];
    if (!tabData || !tabData.queryResult) {
      await showError("No data available");
      return;
    }

    // Show column information
    const columns = tabData.queryResult.columns || [];
    const columnInfo = columns
      .map((col) => `${col.name} (${col.data_type || "unknown"})`)
      .join("\n");
    await showMessage(`Columns:\n\n${columnInfo}`, "Table Columns");
  }

  // Database Menu Handlers
  async function handleConnect() {
    if (!$activeConnection) {
      await showError(
        "No connection selected. Please create a connection first."
      );
      showModal = true;
    } else {
      await showMessage("Already connected to: " + $activeConnection.name);
    }
  }

  async function handleDisconnect() {
    if (!$activeConnection) {
      await showError("No active connection");
      return;
    }

    try {
      const { disconnectFromDatabase } = await import("./utils/tauri");
      await disconnectFromDatabase($activeConnection.id);
      await showMessage("Disconnected successfully");
    } catch (error) {
      console.error("Failed to disconnect:", error);
      await showError("Failed to disconnect: " + error.message);
    }
  }

  // Toolbar Handlers
  async function handleExecute() {
    if (!activeTab || activeTab.type !== "query") {
      await showError("No query tab is active");
      return;
    }

    // Trigger execute in SqlEditor
    document.dispatchEvent(
      new CustomEvent("execute-query", {
        detail: { tabId: activeTab.id },
      })
    );
  }

  async function handleExecuteScript() {
    if (!activeTab || activeTab.type !== "query") {
      await showError("No query tab is active");
      return;
    }

    // Trigger execute script in SqlEditor
    document.dispatchEvent(
      new CustomEvent("execute-script", {
        detail: { tabId: activeTab.id },
      })
    );
  }

  function handleStop() {
    if (!activeTab) {
      return;
    }

    const controller = runningQueries.get(activeTab.id);
    if (controller) {
      controller.abort();
      runningQueries.delete(activeTab.id);

      // Dispatch stop event to SqlEditor
      document.dispatchEvent(
        new CustomEvent("stop-query", {
          detail: { tabId: activeTab.id },
        })
      );
    }
  }

  async function handleCommit() {
    if (!$activeConnection) {
      await showError("No active connection");
      return;
    }

    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("execute_query", {
        connection: $activeConnection,
        query: "COMMIT",
      });
      await showMessage("Transaction committed successfully");
    } catch (error) {
      console.error("Failed to commit transaction:", error);
      await showError("Failed to commit transaction: " + error.message);
    }
  }

  async function handleRollback() {
    if (!$activeConnection) {
      await showError("No active connection");
      return;
    }

    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("execute_query", {
        connection: $activeConnection,
        query: "ROLLBACK",
      });
      await showMessage("Transaction rolled back successfully");
    } catch (error) {
      console.error("Failed to rollback transaction:", error);
      await showError("Failed to rollback transaction: " + error.message);
    }
  }

  async function handleRefresh() {
    if (!activeTab) {
      await showError("No active tab");
      return;
    }

    if (activeTab.type === "table") {
      // Refresh table data
      try {
        const { getTableData } = await import("./utils/tauri");
        const tableInfo = activeTab.tableInfo;

        let tableIdentifier = tableInfo.name;
        if (tableInfo.connection.db_type === "PostgreSQL" && tableInfo.schema) {
          tableIdentifier = `${tableInfo.schema}.${tableInfo.name}`;
        } else if (tableInfo.connection.db_type === "MySQL") {
          tableIdentifier = `${tableInfo.database}.${tableInfo.name}`;
        }

        const tableData = await getTableData(
          tableInfo.connection,
          tableInfo.database,
          tableIdentifier,
          200,
          0
        );

        tabDataStore.setQueryResult(activeTab.id, tableData);
        await showMessage("Table data refreshed");
      } catch (error) {
        console.error("Failed to refresh table data:", error);
        await showError("Failed to refresh: " + error.message);
      }
    } else if (activeTab.type === "query") {
      // Re-execute last query
      const tabData = $tabDataStore[activeTab.id];
      if (tabData && tabData.executedQuery) {
        document.dispatchEvent(
          new CustomEvent("execute-query", {
            detail: { tabId: activeTab.id },
          })
        );
      } else {
        await showError("No executed query to refresh");
      }
    }
  }

  // Help Menu Handlers
  async function handleDocumentation() {
    try {
      const { open } = await import("@tauri-apps/plugin-shell");
      await open("https://github.com/yourusername/rustdbgrid#readme");
    } catch (error) {
      console.error("Failed to open documentation:", error);
      await showMessage(
        "Documentation: https://github.com/yourusername/rustdbgrid"
      );
    }
  }

  function handleAbout() {
    showAboutModal = true;
  }

  function closeAboutModal() {
    showAboutModal = false;
  }

  function handleKeyboardShortcuts() {
    showKeyboardShortcutsModal = true;
  }

  function closeKeyboardShortcutsModal() {
    showKeyboardShortcutsModal = false;
  }

  function addNewQueryTab() {
    const newTab = {
      id: Date.now(),
      title: `Query ${tabs.length + 1}`,
      type: "query",
      modified: false,
    };
    tabs = [...tabs, newTab];
    activeTab = newTab;
  }

  function handleTabSelect(event) {
    activeTab = event.detail;
  }

  function handleTabClose(event) {
    const tabToClose = event.detail;
    const index = tabs.findIndex((t) => t.id === tabToClose.id);

    // Clear tab data from store
    tabDataStore.removeTab(tabToClose.id);

    // Remove tab from list
    tabs = tabs.filter((t) => t.id !== tabToClose.id);

    // If the closed tab was active, switch to another tab
    if (activeTab?.id === tabToClose.id && tabs.length > 0) {
      // Pindah ke tab sebelah kanan jika ada, jika tidak ada pindah ke kiri
      if (index < tabs.length) {
        activeTab = tabs[index]; // Tab di sebelah kanan
      } else {
        activeTab = tabs[index - 1]; // Tab di sebelah kiri
      }
    } else if (tabs.length === 0) {
      activeTab = null;
    }
  }

  function handleNewTab() {
    addNewQueryTab();
  }

  async function handleOpenTableTab(event) {
    const { table, database, connection } = event.detail;

    console.log("Opening table tab:", { table, database, connection });

    // Cek apakah tab untuk tabel ini sudah ada
    const tableFullName =
      connection.db_type === "PostgreSQL" && table.schema
        ? `${table.schema}.${table.name}`
        : table.name;

    const existingTab = tabs.find(
      (t) =>
        t.type === "table" &&
        t.tableInfo?.name === table.name &&
        t.tableInfo?.schema === table.schema &&
        t.tableInfo?.database === database.name
    );

    if (existingTab) {
      // Jika sudah ada, aktifkan tab tersebut
      activeTab = existingTab;
      return;
    }

    // Buat tab baru untuk tabel
    const displayName =
      connection.db_type === "PostgreSQL" && table.schema
        ? `${table.schema}.${table.name}`
        : table.name;

    const newTab = {
      id: Date.now(),
      title: displayName,
      type: "table",
      modified: false,
      tableInfo: {
        name: table.name,
        schema: table.schema,
        database: database.name,
        connection: connection,
      },
    };

    tabs = [...tabs, newTab];
    activeTab = newTab;

    // Load data tabel
    try {
      console.log("Loading table data...");

      // Untuk PostgreSQL, gunakan format schema.table
      // Untuk MySQL, gunakan format database.table untuk cross-database support
      let tableIdentifier = table.name;
      if (connection.db_type === "PostgreSQL" && table.schema) {
        tableIdentifier = `${table.schema}.${table.name}`;
      } else if (connection.db_type === "MySQL") {
        tableIdentifier = `${database.name}.${table.name}`;
      }

      const tableData = await getTableData(
        connection,
        database.name,
        tableIdentifier,
        200,
        0
      );
      console.log("Table data loaded:", tableData);

      // Set query result and the query used
      let tableQuery;
      if (connection.db_type === "PostgreSQL" && table.schema) {
        tableQuery = `SELECT * FROM "${table.schema}"."${table.name}" LIMIT 200`;
      } else if (connection.db_type === "MySQL") {
        // For MySQL, include database.table for cross-database support
        tableQuery = `SELECT * FROM ${database.name}.${table.name} LIMIT 200`;
      } else {
        tableQuery = `SELECT * FROM ${table.name} LIMIT 200`;
      }

      tabDataStore.setQueryResult(newTab.id, tableData);
      tabDataStore.setExecutedQuery(newTab.id, tableQuery);

      // Force reactive update
      tabs = [...tabs];
    } catch (error) {
      console.error("Failed to load table data:", error);
      await showError(`Failed to load table data: ${error.message || error}`);
    }
  }

  function closeModal() {
    showModal = false;
  }

  function handleMouseDown(event) {
    isResizing = true;
    event.preventDefault();
  }

  function handleMouseMove(event) {
    if (!isResizing) return;

    const newWidth = event.clientX;
    if (newWidth >= minSidebarWidth && newWidth <= maxSidebarWidth) {
      sidebarWidth = newWidth;
      // Update normalSidebarWidth only when not maximized
      if (!isWindowMaximized) {
        normalSidebarWidth = newWidth;
      }
    }
  }

  function handleMouseUp() {
    isResizing = false;
    isResizingEditor = false;
  }

  function handleStartResize(event) {
    isResizing = true;
    event.detail.event.preventDefault();
  }

  function toggleSidebar() {
    showSidebar = !showSidebar;
  }
</script>

<SplashScreen
  bind:show={showSplash}
  progress={loadingProgress}
  message={loadingMessage}
/>

<svelte:window on:mousemove={handleMouseMove} on:mouseup={handleMouseUp} />

<MainLayout
  {showToolbar}
  activeTabId={activeTab?.id}
  on:newQuery={handleMenuAction}
  on:openFile={handleMenuAction}
  on:saveQuery={handleMenuAction}
  on:saveAs={handleMenuAction}
  on:export={handleMenuAction}
  on:import={handleMenuAction}
  on:undo={handleMenuAction}
  on:redo={handleMenuAction}
  on:copy={handleMenuAction}
  on:paste={handleMenuAction}
  on:toggleSidebar={handleMenuAction}
  on:toggleToolbar={handleMenuAction}
  on:viewColumns={handleMenuAction}
  on:documentation={handleMenuAction}
  on:about={handleMenuAction}
  on:newConnection={handleMenuAction}
  on:connect={handleMenuAction}
  on:disconnect={handleMenuAction}
  on:execute={handleMenuAction}
  on:executeScript={handleMenuAction}
  on:stop={handleMenuAction}
  on:commit={handleMenuAction}
  on:rollback={handleMenuAction}
  on:refresh={handleMenuAction}
>
  <svelte:fragment slot="sidebar">
    <SidebarWrapper
      bind:show={showSidebar}
      bind:width={sidebarWidth}
      bind:isResizing
      minWidth={minSidebarWidth}
      maxWidth={maxSidebarWidth}
      on:startResize={handleStartResize}
      on:openTableTab={handleOpenTableTab}
    />
  </svelte:fragment>

  <svelte:fragment slot="content">
    <ContentArea
      {tabs}
      {activeTab}
      {currentTabData}
      {editorHeight}
      {isResizingEditor}
      on:tabSelect={handleTabSelect}
      on:tabClose={handleTabClose}
      on:newTab={handleNewTab}
      on:newQuery={addNewQueryTab}
      on:newConnection={() => (showModal = true)}
      on:toggleSidebar={toggleSidebar}
      on:keyboardShortcuts={handleKeyboardShortcuts}
    />
  </svelte:fragment>
</MainLayout>

{#if showModal}
  <ConnectionModal on:close={closeModal} on:save={closeModal} />
{/if}

<AboutModal bind:show={showAboutModal} on:close={closeAboutModal} />

<KeyboardShortcutsModal
  bind:show={showKeyboardShortcutsModal}
  on:close={closeKeyboardShortcutsModal}
/>
