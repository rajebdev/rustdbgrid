<script>
  import { onMount } from "svelte";
  import MenuBar from "./components/MenuBar.svelte";
  import TabBar from "./components/TabBar.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import SqlEditor from "./components/SqlEditor.svelte";
  import DataGrid from "./components/DataGrid.svelte";
  import StatusBar from "./components/StatusBar.svelte";
  import ConnectionModal from "./components/ConnectionModal.svelte";
  import SplashScreen from "./components/SplashScreen.svelte";
  import { activeConnection } from "./stores/connections";
  import { tabDataStore } from "./stores/tabData";
  import { getTableData } from "./utils/tauri";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { listen } from "@tauri-apps/api/event";

  let showSidebar = true;
  let showModal = false;
  let showSplash = true;
  let loadingProgress = 0;
  let loadingMessage = "Initializing application";

  let tabs = [];
  let activeTab = null;

  // Sidebar resize functionality
  let sidebarWidth = 275;
  let minSidebarWidth = 200;
  let maxSidebarWidth = 600;
  let isResizing = false;
  let isWindowMaximized = false;
  let normalSidebarWidth = 275; // Store normal width when not maximized

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
    };
  });

  function handleMenuAction(event) {
    const action = event.type;
    console.log("Menu action:", action);

    switch (action) {
      case "newQuery":
        addNewQueryTab();
        break;
      case "toggleSidebar":
        showSidebar = !showSidebar;
        break;
      case "newConnection":
        showModal = true;
        break;
    }
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
      let tableIdentifier = table.name;
      if (connection.db_type === "PostgreSQL" && table.schema) {
        tableIdentifier = `${table.schema}.${table.name}`;
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
      } else {
        tableQuery = `SELECT * FROM ${table.name} LIMIT 200`;
      }

      tabDataStore.setQueryResult(newTab.id, tableData);
      tabDataStore.setExecutedQuery(newTab.id, tableQuery);

      // Force reactive update
      tabs = [...tabs];
    } catch (error) {
      console.error("Failed to load table data:", error);
      alert(`Failed to load table data: ${error.message || error}`);
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

  function handleEditorMouseDown(event) {
    isResizingEditor = true;
    event.preventDefault();
  }

  function handleEditorMouseMove(event) {
    if (!isResizingEditor) return;

    const container = event.currentTarget;
    const rect = container.getBoundingClientRect();
    const newHeight = event.clientY - rect.top;

    if (newHeight >= minEditorHeight && newHeight <= maxEditorHeight) {
      editorHeight = newHeight;
    }
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

<div class="d-flex flex-column vh-100 overflow-hidden bg-body">
  <MenuBar
    on:newQuery={handleMenuAction}
    on:openFile={handleMenuAction}
    on:saveQuery={handleMenuAction}
    on:saveAs={handleMenuAction}
    on:export={handleMenuAction}
    on:import={handleMenuAction}
    on:toggleSidebar={handleMenuAction}
    on:toggleToolbar={handleMenuAction}
    on:viewColumns={handleMenuAction}
    on:documentation={handleMenuAction}
    on:about={handleMenuAction}
    on:newConnection={handleMenuAction}
    on:execute={handleMenuAction}
    on:executeScript={handleMenuAction}
    on:stop={handleMenuAction}
    on:commit={handleMenuAction}
    on:rollback={handleMenuAction}
    on:refresh={handleMenuAction}
  />

  <div class="d-flex flex-grow-1 overflow-hidden">
    {#if showSidebar}
      <div
        class="sidebar-container border-end"
        style="width: {sidebarWidth}px; flex-shrink: 0; position: relative;"
      >
        <Sidebar
          on:openTableTab={handleOpenTableTab}
          onToggleSidebar={toggleSidebar}
        />
        <button
          class="resize-handle"
          on:mousedown={handleMouseDown}
          aria-label="Resize sidebar"
          class:resizing={isResizing}
        ></button>
      </div>
    {:else}
      <button
        class="sidebar-toggle-button btn btn-sm btn-primary"
        on:click={toggleSidebar}
        title="Show Sidebar (Ctrl+B)"
      >
        <i class="fas fa-chevron-right"></i>
      </button>
    {/if}

    <div
      class="d-flex flex-column flex-grow-1"
      style="min-height: 0; overflow: hidden;"
    >
      <div style="flex-shrink: 0; position: sticky; top: 0; z-index: 100;">
        <TabBar
          {tabs}
          {activeTab}
          on:select={handleTabSelect}
          on:close={handleTabClose}
          on:new={handleNewTab}
        />
      </div>

      <div
        class="flex-grow-1 d-flex flex-column"
        style="overflow: hidden; min-height: 0;"
      >
        {#if activeTab}
          {#if activeTab.type === "query"}
            <div
              class="d-flex flex-column h-100"
              class:resizing={isResizingEditor}
              on:mousemove={handleEditorMouseMove}
              role="presentation"
            >
              <div
                class="border-bottom-2 border-secondary position-relative"
                style="height: {currentTabData?.queryResult
                  ? `${editorHeight}px`
                  : '100%'}; flex-shrink: 0; overflow: hidden;"
              >
                <SqlEditor tabId={activeTab.id} />
              </div>

              {#if currentTabData?.queryResult}
                <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
                <!-- svelte-ignore a11y-no-static-element-interactions -->
                <div
                  class="bg-body-tertiary border-top border-bottom editor-resize-handle"
                  style="height: 6px; cursor: row-resize;"
                  on:mousedown={handleEditorMouseDown}
                  role="separator"
                  aria-orientation="horizontal"
                  aria-label="Resize editor"
                ></div>
                <div class="flex-grow-1 overflow-hidden">
                  <DataGrid
                    data={currentTabData.queryResult}
                    tabId={activeTab.id}
                    executedQuery={currentTabData?.executedQuery || ""}
                    connection={$activeConnection}
                  />
                </div>
              {/if}
            </div>
          {:else if activeTab.type === "table"}
            <div class="flex-grow-1 overflow-hidden">
              {#if currentTabData?.queryResult}
                <DataGrid
                  data={currentTabData.queryResult}
                  tabId={activeTab.id}
                  executedQuery={currentTabData?.executedQuery || ""}
                  connection={activeTab.tableInfo?.connection ||
                    $activeConnection}
                />
              {:else}
                <div
                  class="d-flex flex-column align-items-center justify-content-center h-100 text-secondary gap-3 bg-body-tertiary"
                >
                  <i
                    class="fas fa-spinner fa-spin"
                    style="font-size: 48px; color: #0d6efd;"
                  ></i>
                  <p class="fs-6 m-0 text-dark">Loading table data...</p>
                </div>
              {/if}
            </div>
          {/if}
        {:else}
          <div
            class="d-flex flex-column align-items-center justify-content-center h-100 bg-body-tertiary p-4"
            style="gap: 2rem;"
          >
            <!-- Logo besar -->
            <div class="text-center">
              <i
                class="fas fa-database text-primary"
                style="font-size: 120px; opacity: 0.9;"
              ></i>
              <h2 class="mt-3 fw-bold text-dark">RustDBGrid</h2>
              <p class="text-muted">Universal Database Management Tool</p>
            </div>

            <!-- Start section -->
            <div class="welcome-section">
              <h5 class="text-muted mb-3">
                <i class="fas fa-play-circle me-2"></i>Start
              </h5>
              <div class="d-flex flex-column gap-2">
                <button
                  class="welcome-button btn btn-link text-start d-flex align-items-center gap-3 text-decoration-none px-3 py-2"
                  on:click={addNewQueryTab}
                >
                  <i
                    class="fas fa-file-code text-primary"
                    style="font-size: 20px;"
                  ></i>
                  <div class="flex-grow-1">
                    <div class="text-dark fw-medium">New Query</div>
                    <small class="text-muted">Create a new SQL query tab</small>
                  </div>
                  <kbd class="kbd-shortcut">Ctrl+N</kbd>
                </button>

                <button
                  class="welcome-button btn btn-link text-start d-flex align-items-center gap-3 text-decoration-none px-3 py-2"
                  on:click={() => (showModal = true)}
                >
                  <i class="fas fa-plug text-success" style="font-size: 20px;"
                  ></i>
                  <div class="flex-grow-1">
                    <div class="text-dark fw-medium">New Connection</div>
                    <small class="text-muted">Connect to a database</small>
                  </div>
                  <kbd class="kbd-shortcut">Ctrl+Shift+C</kbd>
                </button>

                <button
                  class="welcome-button btn btn-link text-start d-flex align-items-center gap-3 text-decoration-none px-3 py-2"
                  on:click={() => (showSidebar = !showSidebar)}
                >
                  <i class="fas fa-sidebar text-info" style="font-size: 20px;"
                  ></i>
                  <div class="flex-grow-1">
                    <div class="text-dark fw-medium">Toggle Sidebar</div>
                    <small class="text-muted">Show/hide database explorer</small
                    >
                  </div>
                  <kbd class="kbd-shortcut">Ctrl+B</kbd>
                </button>
              </div>
            </div>

            <!-- Help section -->
            <div class="welcome-section">
              <h5 class="text-muted mb-3">
                <i class="fas fa-question-circle me-2"></i>Help
              </h5>
              <div class="d-flex flex-column gap-2">
                <button
                  class="welcome-button btn btn-link text-start d-flex align-items-center gap-3 text-decoration-none px-3 py-2"
                >
                  <i class="fas fa-book text-warning" style="font-size: 20px;"
                  ></i>
                  <div class="flex-grow-1">
                    <div class="text-dark fw-medium">Documentation</div>
                    <small class="text-muted">Learn how to use RustDBGrid</small
                    >
                  </div>
                </button>

                <button
                  class="welcome-button btn btn-link text-start d-flex align-items-center gap-3 text-decoration-none px-3 py-2"
                >
                  <i
                    class="fas fa-keyboard text-secondary"
                    style="font-size: 20px;"
                  ></i>
                  <div class="flex-grow-1">
                    <div class="text-dark fw-medium">Keyboard Shortcuts</div>
                    <small class="text-muted"
                      >View all available shortcuts</small
                    >
                  </div>
                  <kbd class="kbd-shortcut">Ctrl+K Ctrl+S</kbd>
                </button>
              </div>
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>

  <StatusBar activeTabId={activeTab?.id} />

  {#if showModal}
    <ConnectionModal on:close={closeModal} on:save={closeModal} />
  {/if}
</div>

<style>
  .welcome-section {
    width: 100%;
    max-width: 600px;
  }

  .welcome-button {
    border-radius: 8px;
    transition: all 0.2s ease;
    background-color: transparent;
    border: 1px solid transparent;
  }

  .welcome-button:hover {
    background-color: rgba(0, 0, 0, 0.03);
    border-color: rgba(0, 0, 0, 0.1);
    transform: translateX(4px);
  }

  .kbd-shortcut {
    background-color: #f0f0f0;
    border: 1px solid #ccc;
    border-radius: 4px;
    padding: 2px 8px;
    font-size: 11px;
    font-family: "Consolas", "Monaco", monospace;
    color: #666;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  }

  .sidebar-container {
    position: relative;
  }

  .resize-handle {
    position: absolute;
    top: 0;
    right: 0;
    width: 4px;
    height: 100%;
    cursor: col-resize;
    background-color: transparent;
    transition: background-color 0.2s;
    z-index: 10;
    border: none;
    padding: 0;
  }

  .resize-handle:hover,
  .resize-handle.resizing {
    background-color: #0d6efd;
  }

  .resize-handle::before {
    content: "";
    position: absolute;
    top: 0;
    left: -2px;
    width: 8px;
    height: 100%;
  }

  .sidebar-toggle-button {
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    z-index: 1000;
    border-radius: 0 4px 4px 0;
    padding: 8px 6px;
    font-size: 12px;
    box-shadow: 2px 2px 8px rgba(0, 0, 0, 0.15);
    border-left: none;
  }

  .sidebar-toggle-button:hover {
    padding-left: 8px;
    transition: padding 0.2s ease;
  }

  .editor-resize-handle {
    display: block;
    transition: background-color 0.2s;
  }

  .editor-resize-handle:hover {
    background-color: #0d6efd !important;
  }

  .editor-resize-handle:focus {
    outline: 2px solid #0d6efd;
    outline-offset: -2px;
  }

  .resizing {
    cursor: row-resize !important;
  }

  .resizing :global(*) {
    user-select: none !important;
    pointer-events: none !important;
    cursor: row-resize !important;
  }
</style>
