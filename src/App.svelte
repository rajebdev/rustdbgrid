<script>
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import MainLayout from "./shared/components/layout/MainLayout.svelte";
  import SidebarWrapper from "./shared/components/layout/wrappers/SidebarWrapper.svelte";
  import ContentArea from "./shared/components/layout/wrappers/ContentArea.svelte";
  import ConnectionModal from "./features/connection/components/ConnectionModal.svelte";
  import AboutModal from "./features/settings/components/AboutModal.svelte";
  import KeyboardShortcutsModal from "./shared/components/modals/KeyboardShortcutsModal.svelte";
  import InputModal from "./shared/components/modals/InputModal.svelte";
  import QueryListModal from "./features/query/components/QueryListModal.svelte";
  import UnsavedChangesModal from "./shared/components/modals/UnsavedChangesModal.svelte";
  import { activeConnection } from "./features/connection/stores/connections";
  import { tabDataStore } from "./shared/stores/tabData";
  import { tabStore } from "./shared/stores/tabs";
  import { queryListStore } from "./features/query/stores/queryList";
  import { useKeyboardShortcuts } from "./shared/composables/useKeyboardShortcuts";
  import {
    useWindowResize,
    useEditorResize,
    useSidebarResize,
  } from "./shared/composables/useResize";
  import {
    createMenuHandlers,
    handleOpenTableTab,
    handleOpenProcedureTab,
  } from "./shared/handlers/menuHandlers";
  import { initializeApplication } from "./shared/services/appService";
  import {
    initializeTheme,
    toggleTheme,
  } from "./features/settings/services/themeService";
  import { invoke } from "@tauri-apps/api/core";
  import { message, ask } from "@tauri-apps/plugin-dialog";
  import { saveStatus } from "./features/connection/stores/connections";

  // UI State
  let showSidebar = true;
  let showModal = false;
  let showAboutModal = false;
  let showKeyboardShortcutsModal = false;
  let showQueryListModal = false;
  let showToolbar = true;
  let showRenameModal = false;
  let renameModalTitle = "";
  let renameModalValue = "";
  let renameModalCallback = null;
  let showUnsavedChangesModal = false;
  let unsavedChangesTab = null;

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
  const handleShowQueryList = () => (showQueryListModal = true);

  function handleOpenQueryFromList(event) {
    const query = event.detail;

    // Mark query as used (if it has an ID from localStorage)
    if (query.id && !query.isFile && !query.is_file) {
      queryListStore.markUsed(query.id);
    }

    // Create new query tab with the query content
    tabStore.addQueryTab(get(activeConnection), query.content);

    // Get the newly created tab and update its title
    const newTab = get(tabStore.activeTab);
    if (newTab) {
      // Create updated tab object with new properties
      const updatedTab = {
        ...newTab,
        title: query.title,
        filePath: query.filePath || query.file_path || newTab.filePath,
        modified: false, // File from list is not modified
        initialContent: query.content, // Store initial content
      };

      // Update tab in store
      tabStore.updateTab(updatedTab);
    }

    showQueryListModal = false;
  }

  // Handle open query from QueryListModal window event
  const handleOpenQueryWindowEvent = (event) => {
    handleOpenQueryFromList({ detail: event.detail });
  };

  // Menu handlers
  const menuHandlers = createMenuHandlers({
    tabStore,
    tabDataStore,
    activeConnection,
    setShowToolbar: (val) => (showToolbar = val),
    setShowAboutModal: (val) => (showAboutModal = val),
    runningQueries,
  });

  // Action map for menu events
  const actionHandlers = {
    newQuery: menuHandlers.handleNewQuery,
    openFile: menuHandlers.handleOpenFile,
    openQuery: handleShowQueryList,
    openRecentFile: (event) => menuHandlers.handleOpenRecentFile(event.detail),
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
    newQuery: menuHandlers.handleNewQuery,
    openFile: menuHandlers.handleOpenFile,
    openQuery: handleShowQueryList,
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

    // Validate restored tabs
    tabStore.validateTabs();

    // Auto-load table data for restored table tabs
    // DataGrid will auto-load when it mounts, no pre-fetching needed
    const restoredTabs = get(tabStore);

    // Handle execute query in new tab
    const handleExecuteQueryNewTab = (event) => {
      const newTabData = event.detail;
      tabStore.addQueryTab($activeConnection);
      // Get the newly created tab
      const newTab = get(tabStore);
      if (newTab.length > 0) {
        const lastTab = newTab[newTab.length - 1];
        tabDataStore.setQueryText(lastTab.id, newTabData.queryText);
        tabDataStore.setQueryResult(lastTab.id, newTabData.queryResult);
        tabDataStore.setExecutedQuery(lastTab.id, newTabData.executedQuery);
        tabStore.selectTab(lastTab);
      }
    };

    // Handle save query from editor (Ctrl+S)
    const handleSaveQuery = (event) => {
      menuHandlers.handleSaveQuery();
    };

    window.addEventListener("execute-query-new-tab", handleExecuteQueryNewTab);
    document.addEventListener("save-query", handleSaveQuery);
    window.addEventListener("open-query", handleOpenQueryWindowEvent);

    return () => {
      window.removeEventListener(
        "execute-query-new-tab",
        handleExecuteQueryNewTab
      );
      document.removeEventListener("save-query", handleSaveQuery);
      window.removeEventListener("open-query", handleOpenQueryWindowEvent);
    };
  });

  // Event handlers
  function handleMenuAction(event) {
    const handler = actionHandlers[event.type];
    if (handler) {
      // Pass the event to handler so it can access event.detail if needed
      handler(event);
    } else {
      console.warn("Unknown action:", event.type);
    }
  }

  function handleTabSelect(event) {
    tabStore.selectTab(event.detail);
  }

  function handleTabClose(event) {
    const tab = event.detail;

    // Check if tab has unsaved changes
    if (tab.modified && tab.type === "query") {
      unsavedChangesTab = tab;
      showUnsavedChangesModal = true;
    } else {
      // No unsaved changes, close directly
      tabDataStore.removeTab(tab.id);
      tabStore.closeTab(tab);
    }
  }

  async function handleUnsavedChangesSave() {
    showUnsavedChangesModal = false;
    if (unsavedChangesTab) {
      // Save the query first
      const currentActive = get(activeTab);
      tabStore.selectTab(unsavedChangesTab);
      await menuHandlers.handleSaveQuery();

      // Then close the tab
      tabDataStore.removeTab(unsavedChangesTab.id);
      tabStore.closeTab(unsavedChangesTab);
      unsavedChangesTab = null;
    }
  }

  function handleUnsavedChangesDiscard() {
    showUnsavedChangesModal = false;
    if (unsavedChangesTab) {
      // Close without saving
      tabDataStore.removeTab(unsavedChangesTab.id);
      tabStore.closeTab(unsavedChangesTab);
      unsavedChangesTab = null;
    }
  }

  function handleUnsavedChangesCancel() {
    showUnsavedChangesModal = false;
    unsavedChangesTab = null;
  }

  async function handleNewScript(event) {
    // Create a new query tab
    await menuHandlers.handleNewQuery();
  }

  async function handleRevealInExplorer(event) {
    const tab = event.detail;
    if (tab && tab.filePath) {
      try {
        // Convert relative path to absolute if needed
        let fullPath = tab.filePath;
        if (!fullPath.match(/^[a-zA-Z]:[\\\\/]/) && !fullPath.startsWith("/")) {
          // Relative path, make it absolute
          const configDir = await invoke("get_config_dir");
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

        await invoke("open_path_in_explorer", { path: fullPath });
      } catch (error) {
        console.error("Failed to reveal file in explorer:", error);
      }
    }
  }

  async function handleDeleteScript(event) {
    const tab = event.detail;
    if (tab && tab.filePath) {
      try {
        const confirmed = await ask(`Delete "${tab.title}"?`, {
          title: "Confirm Delete",
          kind: "warning",
        });

        if (confirmed) {
          await invoke("delete_file", { path: tab.filePath });

          // Close the tab
          tabDataStore.removeTab(tab.id);
          tabStore.closeTab(tab);

          await message("File deleted successfully", { kind: "info" });
        }
      } catch (error) {
        console.error("Failed to delete file:", error);
        await message("Failed to delete file: " + error.message, {
          kind: "error",
        });
      }
    }
  }

  async function handleRenameFile(event) {
    const tab = event.detail;
    if (tab) {
      renameModalTitle = "Rename File";
      renameModalValue = tab.title;
      renameModalCallback = async (newName) => {
        if (newName && newName !== tab.title) {
          try {
            // If file has been saved to disk, rename the file
            if (tab.filePath) {
              // Get the directory path
              const lastSlash = tab.filePath.lastIndexOf("/");
              const lastBackslash = tab.filePath.lastIndexOf("\\");
              const separatorIndex = Math.max(lastSlash, lastBackslash);
              const directory = tab.filePath.substring(0, separatorIndex);

              const newFilePath = directory + "/" + newName;

              await invoke("rename_file", {
                oldPath: tab.filePath,
                newPath: newFilePath,
              });

              // Update file path
              tab.filePath = newFilePath;
            }

            // Update tab title (always, whether saved or not)
            tab.title = newName;
            tabStore.updateTab(tab);

            // Show success in status bar
            const msgText = tab.filePath
              ? "File renamed successfully"
              : "Tab title updated";
            saveStatus.set({
              message: msgText,
              type: "success",
              timestamp: Date.now(),
            });

            // Clear status after 3 seconds
            setTimeout(() => {
              saveStatus.set({ message: null, type: null, timestamp: null });
            }, 3000);
          } catch (error) {
            console.error("Failed to rename file:", error);

            // Show error in status bar
            saveStatus.set({
              message: `Rename failed: ${error.message}`,
              type: "error",
              timestamp: Date.now(),
            });

            // Clear error after 5 seconds
            setTimeout(() => {
              saveStatus.set({ message: null, type: null, timestamp: null });
            }, 5000);
          }
        }
      };
      showRenameModal = true;
    }
  }

  function handleRenameSubmit(event) {
    const newName = event.detail;
    if (renameModalCallback) {
      renameModalCallback(newName).catch((error) => {
        console.error("Failed to rename:", error);
      });
    }
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
  on:openQuery={handleMenuAction}
  on:openRecentFile={handleMenuAction}
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
        handleOpenTableTab(e, tabStore, tabDataStore);
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
      on:newQuery={menuHandlers.handleNewQuery}
      on:newScript={handleNewScript}
      on:revealInExplorer={handleRevealInExplorer}
      on:copyFilePath={() => {}}
      on:deleteScript={handleDeleteScript}
      on:renameFile={handleRenameFile}
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

<QueryListModal
  bind:show={showQueryListModal}
  on:open={handleOpenQueryFromList}
  on:close={() => (showQueryListModal = false)}
/>

<InputModal
  bind:show={showRenameModal}
  title={renameModalTitle}
  label="Enter new filename:"
  value={renameModalValue}
  placeholder="Filename"
  on:submit={handleRenameSubmit}
  on:cancel={() => (showRenameModal = false)}
/>

<UnsavedChangesModal
  bind:show={showUnsavedChangesModal}
  tabTitle={unsavedChangesTab?.title || ""}
  on:save={handleUnsavedChangesSave}
  on:discard={handleUnsavedChangesDiscard}
  on:cancel={handleUnsavedChangesCancel}
/>
