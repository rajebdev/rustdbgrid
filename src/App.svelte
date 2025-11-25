<script>
  import MenuBar from "./components/MenuBar.svelte";
  import TabBar from "./components/TabBar.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import SqlEditor from "./components/SqlEditor.svelte";
  import DataGrid from "./components/DataGrid.svelte";
  import StatusBar from "./components/StatusBar.svelte";
  import ConnectionModal from "./components/ConnectionModal.svelte";
  import { activeConnection } from "./stores/connections";
  import { tabDataStore } from "./stores/tabData";
  import { getTableData } from "./utils/tauri";

  let showSidebar = true;
  let showModal = false;

  let tabs = [];
  let activeTab = null;

  $: currentTabData = activeTab ? $tabDataStore[activeTab.id] : null;

  // Debug log
  $: if (activeTab) {
    console.log("Active tab changed:", activeTab);
    console.log("Current tab data:", currentTabData);
  }

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
    const existingTab = tabs.find(
      (t) =>
        t.type === "table" &&
        t.tableInfo?.name === table.name &&
        t.tableInfo?.database === database.name
    );

    if (existingTab) {
      // Jika sudah ada, aktifkan tab tersebut
      activeTab = existingTab;
      return;
    }

    // Buat tab baru untuk tabel
    const newTab = {
      id: Date.now(),
      title: table.name,
      type: "table",
      modified: false,
      tableInfo: {
        name: table.name,
        database: database.name,
        connection: connection,
      },
    };

    tabs = [...tabs, newTab];
    activeTab = newTab;

    // Load data tabel
    try {
      console.log("Loading table data...");
      const tableData = await getTableData(
        connection,
        database.name,
        table.name,
        100,
        0
      );
      console.log("Table data loaded:", tableData);

      // Set query result and the query used
      const tableQuery = `SELECT * FROM ${database.name}.${table.name} LIMIT 100`;
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
</script>

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
      <div class="border-end" style="width: 250px; flex-shrink: 0;">
        <Sidebar on:openTableTab={handleOpenTableTab} />
      </div>
    {/if}

    <div
      class="d-flex flex-column flex-grow-1 overflow-hidden bg-body-secondary"
    >
      <TabBar
        {tabs}
        {activeTab}
        on:select={handleTabSelect}
        on:close={handleTabClose}
        on:new={handleNewTab}
      />

      <div class="flex-grow-1 overflow-hidden d-flex flex-column">
        {#if activeTab}
          {#if activeTab.type === "query"}
            <div class="d-flex flex-column h-100">
              <div
                class="border-bottom-2 border-secondary"
                style="height: 300px; flex-shrink: 0; overflow: hidden;"
              >
                <SqlEditor tabId={activeTab.id} />
              </div>

              {#if currentTabData?.queryResult}
                <div
                  class="bg-body-tertiary border-top border-bottom"
                  style="height: 6px; cursor: row-resize;"
                ></div>
                <div class="flex-grow-1 overflow-hidden">
                  <DataGrid
                    data={currentTabData.queryResult}
                    tabId={activeTab.id}
                    executedQuery={currentTabData?.executedQuery || ""}
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
            class="d-flex flex-column align-items-center justify-content-center h-100 text-secondary gap-4 bg-body-tertiary"
          >
            <i
              class="fas fa-file-code opacity-25 text-muted"
              style="font-size: 72px;"
            ></i>
            <p class="fs-5 m-0 text-dark">No tab selected</p>
            <button
              class="btn btn-primary d-flex align-items-center gap-2"
              on:click={addNewQueryTab}
            >
              <i class="fas fa-plus"></i> New Query
            </button>
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
  /* Hover effect for resizer */
  .bg-body-tertiary:hover {
    background-color: #0d6efd !important;
  }
</style>
