<script>
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import MainLayout from "./components/layout/MainLayout.svelte";
  import SidebarWrapper from "./components/layout/wrappers/SidebarWrapper.svelte";
  import ContentArea from "./components/layout/wrappers/ContentArea.svelte";
  import ConnectionModal from "./components/modals/ConnectionModal.svelte";
  import SplashScreen from "./components/screens/SplashScreen.svelte";
  import AboutModal from "./components/modals/AboutModal.svelte";
  import KeyboardShortcutsModal from "./components/modals/KeyboardShortcutsModal.svelte";
  import { activeConnection } from "./stores/connections";
  import { tabDataStore } from "./stores/tabData";
  import { tabStore } from "./stores/tabs";
  import { getTableData } from "./utils/tauri";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { listen } from "@tauri-apps/api/event";
  import { useKeyboardShortcuts } from "./composables/useKeyboardShortcuts";
  import { createMenuHandlers } from "./handlers/menuHandlers";
  import { showMessage, showError } from "./services/fileService";

  // UI State
  let showSidebar = true;
  let showModal = false;
  let showAboutModal = false;
  let showKeyboardShortcutsModal = false;
  let showToolbar = true;
  let showSplash = true;
  let loadingProgress = 0;
  let loadingMessage = "Initializing application";

  // Sidebar resize
  let sidebarWidth = 320;
  let minSidebarWidth = 200;
  let maxSidebarWidth = 600;
  let isResizing = false;
  let isWindowMaximized = false;
  let normalSidebarWidth = 320;

  // Editor resize
  let editorHeight = 300;
  let minEditorHeight = 150;
  let maxEditorHeight = 700;
  let isResizingEditor = false;

  // Query execution tracking
  let runningQueries = new Map();

  // Tab store subscriptions
  let tabs = [];
  let activeTab = null;
  
  $: tabs = $tabStore;
  $: activeTab = $tabStore.activeTab;
  $: currentTabData = activeTab ? $tabDataStore[activeTab.id] : null;

  // Create menu handlers with context
  const menuHandlers = createMenuHandlers({
    get tabs() { return get(tabStore); },
    get activeTab() { return get(tabStore.activeTab); },
    tabDataStore,
    addNewQueryTab: () => tabStore.addQueryTab(),
    get activeConnection() { return get(activeConnection); },
    get showModal() { return showModal; },
    setShowModal: (val) => showModal = val,
    get showSidebar() { return showSidebar; },
    get showToolbar() { return showToolbar; },
    setShowToolbar: (val) => showToolbar = val,
    get showAboutModal() { return showAboutModal; },
    setShowAboutModal: (val) => showAboutModal = val,
    runningQueries,
    getTableData,
    setActiveTab: (tab) => tabStore.selectTab(tab),
    updateTabs: () => tabStore.updateTabs(),
  });

  // Setup keyboard shortcuts
  useKeyboardShortcuts({
    newQuery: () => tabStore.addQueryTab(),
    openFile: menuHandlers.handleOpenFile,
    saveQuery: menuHandlers.handleSaveQuery,
    saveAs: menuHandlers.handleSaveAs,
    toggleSidebar: () => showSidebar = !showSidebar,
    newConnection: () => showModal = true,
    execute: menuHandlers.handleExecute,
    executeScript: menuHandlers.handleExecuteScript,
    refresh: menuHandlers.handleRefresh,
    closeTab: () => {
      if (activeTab && tabs.length > 0) {
        handleTabClose({ detail: activeTab });
      }
    },
    nextTab: () => tabStore.nextTab(),
    previousTab: () => tabStore.previousTab(),
    showKeyboardShortcuts: () => showKeyboardShortcutsModal = true,
  });

  onMount(async () => {
    // Window maximize/unmaximize listener
    const appWindow = getCurrentWindow();
    isWindowMaximized = await appWindow.isMaximized();
    
    if (isWindowMaximized) {
      normalSidebarWidth = sidebarWidth;
      sidebarWidth = 320;
    }

    const unlisten = await listen("tauri://resize", async () => {
      const maximized = await appWindow.isMaximized();
      if (maximized !== isWindowMaximized) {
        isWindowMaximized = maximized;
        if (maximized) {
          normalSidebarWidth = sidebarWidth;
          sidebarWidth = 320;
        } else {
          sidebarWidth = normalSidebarWidth;
        }
      }
    });

    // Editor resize listeners
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

    // Splash screen initialization
    await initializeApp();

    return () => {
      unlisten();
      window.removeEventListener("start-editor-resize", handleStartEditorResize);
      window.removeEventListener("editor-resize-move", handleEditorResizeMove);
    };
  });

  async function initializeApp() {
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

    await new Promise((resolve) => setTimeout(resolve, 300));
    showSplash = false;
  }

  function handleMenuAction(event) {
    const action = event.type;
    const actionMap = {
      // File Menu
      newQuery: () => tabStore.addQueryTab(),
      openFile: menuHandlers.handleOpenFile,
      saveQuery: menuHandlers.handleSaveQuery,
      saveAs: menuHandlers.handleSaveAs,
      export: menuHandlers.handleExportData,
      import: menuHandlers.handleImportData,
      
      // Edit Menu
      undo: menuHandlers.handleUndo,
      redo: menuHandlers.handleRedo,
      copy: menuHandlers.handleCopy,
      paste: menuHandlers.handlePaste,
      
      // View Menu
      toggleSidebar: () => showSidebar = !showSidebar,
      toggleToolbar: menuHandlers.handleToggleToolbar,
      viewColumns: menuHandlers.handleViewColumns,
      
      // Database Menu
      newConnection: () => showModal = true,
      connect: menuHandlers.handleConnect,
      disconnect: menuHandlers.handleDisconnect,
      
      // Toolbar Actions
      execute: menuHandlers.handleExecute,
      executeScript: menuHandlers.handleExecuteScript,
      stop: menuHandlers.handleStop,
      commit: menuHandlers.handleCommit,
      rollback: menuHandlers.handleRollback,
      refresh: menuHandlers.handleRefresh,
      
      // Help Menu
      documentation: menuHandlers.handleDocumentation,
      about: menuHandlers.handleAbout,
    };

    const handler = actionMap[action];
    if (handler) {
      handler();
    } else {
      console.warn("Unknown action:", action);
    }
  }

  function handleTabSelect(event) {
    tabStore.selectTab(event.detail);
  }

  function handleTabClose(event) {
    const tabToClose = event.detail;
    tabDataStore.removeTab(tabToClose.id);
    tabStore.closeTab(tabToClose);
  }

  function handleNewTab() {
    tabStore.addQueryTab();
  }

  async function handleOpenTableTab(event) {
    const { table, database, connection } = event.detail;
    
    tabStore.addTableTab(table, database, connection);
    
    // Get the newly created tab
    const newTab = get(tabStore.activeTab);
    
    if (!newTab) return;

    // Load table data
    try {
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

      let tableQuery;
      if (connection.db_type === "PostgreSQL" && table.schema) {
        tableQuery = `SELECT * FROM "${table.schema}"."${table.name}" LIMIT 200`;
      } else if (connection.db_type === "MySQL") {
        tableQuery = `SELECT * FROM ${database.name}.${table.name} LIMIT 200`;
      } else {
        tableQuery = `SELECT * FROM ${table.name} LIMIT 200`;
      }

      tabDataStore.setQueryResult(newTab.id, tableData);
      tabDataStore.setExecutedQuery(newTab.id, tableQuery);
    } catch (error) {
      console.error("Failed to load table data:", error);
      await showError(`Failed to load table data: ${error.message || error}`);
    }
  }

  function handleMouseMove(event) {
    if (!isResizing) return;
    const newWidth = event.clientX;
    if (newWidth >= minSidebarWidth && newWidth <= maxSidebarWidth) {
      sidebarWidth = newWidth;
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
      on:newQuery={() => tabStore.addQueryTab()}
      on:newConnection={() => showModal = true}
      on:toggleSidebar={() => showSidebar = !showSidebar}
      on:keyboardShortcuts={() => showKeyboardShortcutsModal = true}
    />
  </svelte:fragment>
</MainLayout>

{#if showModal}
  <ConnectionModal on:close={() => showModal = false} on:save={() => showModal = false} />
{/if}

<AboutModal bind:show={showAboutModal} on:close={() => showAboutModal = false} />

<KeyboardShortcutsModal
  bind:show={showKeyboardShortcutsModal}
  on:close={() => showKeyboardShortcutsModal = false}
/>
