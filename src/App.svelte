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
  import { useKeyboardShortcuts } from "./composables/useKeyboardShortcuts";
  import {
    useWindowResize,
    useEditorResize,
    useSidebarResize,
  } from "./composables/useResize";
  import {
    createMenuHandlers,
    handleOpenTableTab,
  } from "./handlers/menuHandlers";
  import { initializeApplication } from "./services/appService";

  // UI State
  let showSidebar = true;
  let showModal = false;
  let showAboutModal = false;
  let showKeyboardShortcutsModal = false;
  let showToolbar = true;
  let showSplash = true;
  let loadingProgress = 0;
  let loadingMessage = "Initializing...";

  // Resize state
  let sidebarWidth = 320;
  let normalSidebarWidth = 320;
  let isResizing = false;
  let editorHeight = 300;
  let isResizingEditor = false;
  let runningQueries = new Map();

  // Tab subscriptions
  $: tabs = $tabStore;
  $: activeTab = $tabStore.activeTab;
  $: currentTabData = activeTab ? $tabDataStore[activeTab.id] : null;

  // Menu handlers
  const menuHandlers = createMenuHandlers({
    get tabs() {
      return get(tabStore);
    },
    get activeTab() {
      return get(tabStore.activeTab);
    },
    tabDataStore,
    addNewQueryTab: () => tabStore.addQueryTab(),
    get activeConnection() {
      return get(activeConnection);
    },
    setShowModal: (val) => (showModal = val),
    setShowToolbar: (val) => (showToolbar = val),
    setShowAboutModal: (val) => (showAboutModal = val),
    runningQueries,
    getTableData,
    setActiveTab: (tab) => tabStore.selectTab(tab),
    updateTabs: () => tabStore.updateTabs(),
  });

  // Action map for menu events
  const actionHandlers = {
    newQuery: () => tabStore.addQueryTab(),
    openFile: menuHandlers.handleOpenFile,
    saveQuery: menuHandlers.handleSaveQuery,
    saveAs: menuHandlers.handleSaveAs,
    export: menuHandlers.handleExportData,
    import: menuHandlers.handleImportData,
    undo: menuHandlers.handleUndo,
    redo: menuHandlers.handleRedo,
    copy: menuHandlers.handleCopy,
    paste: menuHandlers.handlePaste,
    toggleSidebar: () => (showSidebar = !showSidebar),
    toggleToolbar: menuHandlers.handleToggleToolbar,
    viewColumns: menuHandlers.handleViewColumns,
    newConnection: () => (showModal = true),
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
  useKeyboardShortcuts({
    newQuery: () => tabStore.addQueryTab(),
    openFile: menuHandlers.handleOpenFile,
    saveQuery: menuHandlers.handleSaveQuery,
    saveAs: menuHandlers.handleSaveAs,
    toggleSidebar: () => (showSidebar = !showSidebar),
    newConnection: () => (showModal = true),
    execute: menuHandlers.handleExecute,
    executeScript: menuHandlers.handleExecuteScript,
    refresh: menuHandlers.handleRefresh,
    closeTab: () =>
      activeTab && tabs.length > 0 && handleTabClose({ detail: activeTab }),
    nextTab: () => tabStore.nextTab(),
    previousTab: () => tabStore.previousTab(),
    showKeyboardShortcuts: () => (showKeyboardShortcutsModal = true),
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
    await initializeApplication(({ progress, message }) => {
      loadingProgress = progress;
      loadingMessage = message;
    });
    showSplash = false;
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
      minWidth={200}
      maxWidth={600}
      on:startResize={handleStartResize}
      on:openTableTab={(e) =>
        handleOpenTableTab(e, tabStore, tabDataStore, getTableData)}
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
      on:newTab={() => tabStore.addQueryTab()}
      on:newQuery={() => tabStore.addQueryTab()}
      on:newConnection={() => (showModal = true)}
      on:toggleSidebar={() => (showSidebar = !showSidebar)}
      on:keyboardShortcuts={() => (showKeyboardShortcutsModal = true)}
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
