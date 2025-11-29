<script>
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import MainLayout from "./components/layout/MainLayout.svelte";
  import SidebarWrapper from "./components/layout/wrappers/SidebarWrapper.svelte";
  import ContentArea from "./components/layout/wrappers/ContentArea.svelte";
  import ConnectionModal from "./components/modals/ConnectionModal.svelte";
  import AboutModal from "./components/modals/AboutModal.svelte";
  import KeyboardShortcutsModal from "./components/modals/KeyboardShortcutsModal.svelte";
  import { activeConnection } from "./stores/connections";
  import { tabDataStore } from "./stores/tabData";
  import { tabStore } from "./stores/tabs";
  import { getTableData } from "./utils/tauri";
  import { useKeyboardShortcuts } from "./composables/useKeyboardShortcuts";
  import {
    useWindowResize,
    useEditorResize,
    useSidebarResize,
  } from "./composables/useResize";
  import {
    createMenuHandlers,
    handleOpenTableTab,
    handleOpenProcedureTab,
  } from "./handlers/menuHandlers";
  import { initializeApplication } from "./services/appService";
  import { initializeTheme, toggleTheme } from "./services/themeService";

  // UI State
  let showSidebar = true;
  let showModal = false;
  let showAboutModal = false;
  let showKeyboardShortcutsModal = false;
  let showToolbar = true;

  // Resize state
  let sidebarWidth = 320;
  let normalSidebarWidth = 320;
  let isResizing = false;
  let editorHeight = 300;
  let isResizingEditor = false;
  let runningQueries = new Map();

  // Derived state from stores
  $: tabs = $tabStore;
  $: activeTab = tabStore.activeTab;
  $: currentTabData = $activeTab ? $tabDataStore[$activeTab.id] : null;

  // Reusable event handlers
  const handleAddQueryTab = () => tabStore.addQueryTab($activeConnection);
  const handleToggleSidebar = () => (showSidebar = !showSidebar);
  const handleShowModal = () => (showModal = true);
  const handleShowKeyboardShortcuts = () => (showKeyboardShortcutsModal = true);

  // Menu handlers
  const menuHandlers = createMenuHandlers({
    tabStore,
    tabDataStore,
    activeConnection,
    addNewQueryTab: handleAddQueryTab,
    setShowModal: (val) => (showModal = val),
    setShowToolbar: (val) => (showToolbar = val),
    setShowAboutModal: (val) => (showAboutModal = val),
    runningQueries,
    getTableData,
  });

  // Action map for menu events
  const actionHandlers = {
    newQuery: handleAddQueryTab,
    openFile: menuHandlers.handleOpenFile,
    saveQuery: menuHandlers.handleSaveQuery,
    saveAs: menuHandlers.handleSaveAs,
    export: menuHandlers.handleExportData,
    import: menuHandlers.handleImportData,
    undo: menuHandlers.handleUndo,
    redo: menuHandlers.handleRedo,
    copy: menuHandlers.handleCopy,
    paste: menuHandlers.handlePaste,
    toggleSidebar: handleToggleSidebar,
    toggleToolbar: menuHandlers.handleToggleToolbar,
    viewColumns: menuHandlers.handleViewColumns,
    newConnection: handleShowModal,
    connect: menuHandlers.handleConnect,
    disconnect: menuHandlers.handleDisconnect,
    execute: menuHandlers.handleExecute,
    executeScript: menuHandlers.handleExecuteScript,
    stop: menuHandlers.handleStop,
    commit: menuHandlers.handleCommit,
    rollback: menuHandlers.handleRollback,
    refresh: menuHandlers.handleRefresh,
    documentation: menuHandlers.handleDocumentation,
    about: menuHandlers.handleAbout,
  };

  // Keyboard shortcuts
  const handleCloseActiveTab = () => {
    if ($activeTab && tabs.length > 0) {
      handleTabClose({ detail: $activeTab });
    }
  };

  useKeyboardShortcuts({
    newQuery: handleAddQueryTab,
    openFile: menuHandlers.handleOpenFile,
    saveQuery: menuHandlers.handleSaveQuery,
    saveAs: menuHandlers.handleSaveAs,
    toggleSidebar: handleToggleSidebar,
    newConnection: handleShowModal,
    execute: menuHandlers.handleExecute,
    executeScript: menuHandlers.handleExecuteScript,
    refresh: menuHandlers.handleRefresh,
    closeTab: handleCloseActiveTab,
    nextTab: () => tabStore.nextTab(),
    previousTab: () => tabStore.previousTab(),
    showKeyboardShortcuts: handleShowKeyboardShortcuts,
    toggleTheme: toggleTheme,
  });

  // Window resize handling
  useWindowResize((isMaximized) => {
    if (isMaximized) {
      normalSidebarWidth = sidebarWidth;
      sidebarWidth = 320;
    } else {
      sidebarWidth = normalSidebarWidth;
    }
  });

  // Editor resize handling
  useEditorResize({
    initialHeight: editorHeight,
    minHeight: 150,
    maxHeight: 700,
    onResizeStart: () => (isResizingEditor = true),
    onResize: (height) => (editorHeight = height),
    onResizeEnd: () => (isResizingEditor = false),
  });

  // Sidebar resize
  const sidebarResize = useSidebarResize({
    minWidth: 200,
    maxWidth: 600,
  });

  onMount(async () => {
    // Initialize theme system first
    await initializeTheme();

    await initializeApplication();
  });

  // Event handlers
  function handleMenuAction(event) {
    const handler = actionHandlers[event.type];
    handler ? handler() : console.warn("Unknown action:", event.type);
  }

  function handleTabSelect(event) {
    tabStore.selectTab(event.detail);
  }

  function handleTabClose(event) {
    tabDataStore.removeTab(event.detail.id);
    tabStore.closeTab(event.detail);
  }

  function handleMouseMove(event) {
    const newWidth = sidebarResize.handleMouseMove(event, isResizing);
    if (newWidth !== null) {
      sidebarWidth = newWidth;
      if (!isResizing) normalSidebarWidth = newWidth;
    }
  }

  function handleMouseUp() {
    isResizing = false;
  }

  function handleStartResize(event) {
    isResizing = true;
    event.detail.event.preventDefault();
  }
</script>

<svelte:window on:mousemove={handleMouseMove} on:mouseup={handleMouseUp} />

<MainLayout
  {showToolbar}
  activeTabId={$activeTab?.id}
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
      minWidth={200}
      maxWidth={600}
      on:startResize={handleStartResize}
      on:openTableTab={(e) => {
        handleOpenTableTab(e, tabStore, tabDataStore, getTableData);
      }}
      on:openProcedureTab={(e) => {
        handleOpenProcedureTab(e, tabStore, tabDataStore);
      }}
    />
  </svelte:fragment>

  <svelte:fragment slot="content">
    <ContentArea
      {tabs}
      activeTab={$activeTab}
      {currentTabData}
      {editorHeight}
      {isResizingEditor}
      on:tabSelect={handleTabSelect}
      on:tabClose={handleTabClose}
      on:newTab={handleAddQueryTab}
      on:newQuery={handleAddQueryTab}
      on:newConnection={handleShowModal}
      on:toggleSidebar={handleToggleSidebar}
      on:keyboardShortcuts={handleShowKeyboardShortcuts}
    />
  </svelte:fragment>
</MainLayout>

{#if showModal}
  <ConnectionModal
    on:close={() => (showModal = false)}
    on:save={() => (showModal = false)}
  />
{/if}

<AboutModal
  bind:show={showAboutModal}
  on:close={() => (showAboutModal = false)}
/>

<KeyboardShortcutsModal
  bind:show={showKeyboardShortcutsModal}
  on:close={() => (showKeyboardShortcutsModal = false)}
/>
