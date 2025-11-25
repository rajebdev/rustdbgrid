<script>
  import { onMount, createEventDispatcher } from "svelte";
  import {
    connections,
    activeConnection,
    selectedDatabase,
    selectedTable,
  } from "../stores/connections";
  import {
    getConnections,
    getDatabases,
    getTables,
    getStorageInfo,
  } from "../utils/tauri";

  const dispatch = createEventDispatcher();
  import ConnectionModal from "./ConnectionModal.svelte";

  let databases = [];
  let tables = [];
  let showModal = false;
  let editingConnection = null;
  let expandedConnections = {};
  let expandedDatabases = {};
  let expandedTables = {};
  let searchQuery = "";
  let storageInfo = null;

  onMount(async () => {
    await loadConnections();
    await loadStorageInfo();
  });

  async function loadConnections() {
    try {
      const conns = await getConnections();
      connections.set(conns);
    } catch (error) {
      console.error("Failed to load connections:", error);
    }
  }

  async function loadStorageInfo() {
    try {
      storageInfo = await getStorageInfo();
    } catch (error) {
      console.error("Failed to load storage info:", error);
    }
  }

  async function toggleConnection(conn) {
    const isExpanded = expandedConnections[conn.id];

    if (!isExpanded) {
      activeConnection.set(conn);
      try {
        databases = await getDatabases(conn);
        expandedConnections[conn.id] = { databases };
      } catch (error) {
        console.error("Failed to load databases:", error);
      }
    } else {
      delete expandedConnections[conn.id];
    }
    expandedConnections = { ...expandedConnections };
  }

  async function toggleDatabase(connId, db) {
    const key = `${connId}-${db.name}`;
    const isExpanded = expandedDatabases[key];

    if (!isExpanded) {
      selectedDatabase.set(db);
      try {
        const dbTables = await getTables($activeConnection, db.name);
        expandedDatabases[key] = { tables: dbTables };
      } catch (error) {
        console.error("Failed to load tables:", error);
      }
    } else {
      delete expandedDatabases[key];
    }
    expandedDatabases = { ...expandedDatabases };
  }

  function toggleTables(connId, dbName) {
    const key = `${connId}-${dbName}`;
    if (expandedTables[key]) {
      delete expandedTables[key];
    } else {
      expandedTables[key] = true;
    }
    expandedTables = { ...expandedTables };
  }

  function selectTable(table) {
    selectedTable.set(table);
  }

  function handleTableDoubleClick(table) {
    // Dispatch event untuk membuka tab baru dengan data tabel
    dispatch("openTableTab", {
      table,
      database: $selectedDatabase,
      connection: $activeConnection,
    });
  }

  function formatBytes(bytes) {
    if (!bytes || bytes === 0) return "";

    const k = 1024;
    const sizes = ["B", "K", "M", "G", "T"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));

    const value = bytes / Math.pow(k, i);

    // Format: semua angka dibulatkan tanpa desimal
    const formatted = Math.round(value);

    return `${formatted}${sizes[i]}`;
  }

  function openNewConnectionModal() {
    editingConnection = null;
    showModal = true;
  }

  function closeModal() {
    showModal = false;
    editingConnection = null;
  }

  async function handleSaveConnection() {
    closeModal();
    await loadConnections();
    await loadStorageInfo();
  }

  $: filteredConnections = $connections.filter((conn) =>
    conn.name.toLowerCase().includes(searchQuery.toLowerCase())
  );
</script>

<div class="d-flex flex-column h-100 bg-body-tertiary text-dark">
  <div class="p-3 pb-2 border-bottom bg-body">
    <h6
      class="text-uppercase text-secondary mb-2"
      style="font-size: 11px; font-weight: 600; letter-spacing: 0.5px;"
    >
      <i class="fas fa-network-wired me-2"></i>
      Connections
    </h6>
    <div class="d-flex gap-2">
      <input
        type="search"
        class="form-control form-control-sm flex-grow-1"
        placeholder="Search connection or database"
        bind:value={searchQuery}
        style="font-size: 12px;"
      />
      <button
        class="btn btn-sm btn-success"
        on:click={openNewConnectionModal}
        style="font-size: 12px; padding: 4px 8px;"
        title="Add Connection"
      >
        <i class="fas fa-plus"></i>
      </button>
    </div>
  </div>

  <div class="flex-grow-1 overflow-auto p-1" style="scrollbar-width: thin;">
    {#if filteredConnections.length === 0}
      <p class="text-muted small p-3">No connections found</p>
    {/if}

    {#each filteredConnections as conn (conn.id)}
      <div class="tree-item">
        <div class="tree-node connection-node">
          <button
            class="tree-toggle"
            on:click={() => toggleConnection(conn)}
            aria-label="Toggle connection"
          >
            <i
              class="fas fa-chevron-{expandedConnections[conn.id]
                ? 'down'
                : 'right'}"
            ></i>
          </button>
          <button
            class="tree-label"
            class:active={$activeConnection?.id === conn.id}
            on:click={() => toggleConnection(conn)}
          >
            <i class="fas fa-server tree-icon"></i>
            <span class="tree-text">{conn.name}</span>
            <span class="tree-badge">{conn.db_type}</span>
          </button>
        </div>

        {#if expandedConnections[conn.id]}
          <div class="tree-children">
            {#each expandedConnections[conn.id].databases || [] as db (db.name)}
              <div class="tree-item">
                <div class="tree-node database-node">
                  <button
                    class="tree-toggle"
                    on:click={() => toggleDatabase(conn.id, db)}
                    aria-label="Toggle database"
                  >
                    <i
                      class="fas fa-chevron-{expandedDatabases[
                        `${conn.id}-${db.name}`
                      ]
                        ? 'down'
                        : 'right'}"
                    ></i>
                  </button>
                  <button
                    class="tree-label"
                    class:active={$selectedDatabase?.name === db.name}
                    on:click={() => toggleDatabase(conn.id, db)}
                  >
                    <i class="fas fa-database tree-icon"></i>
                    <span class="tree-text">{db.name}</span>
                  </button>
                </div>

                {#if expandedDatabases[`${conn.id}-${db.name}`]}
                  <div class="tree-children">
                    <div class="tree-item">
                      <div class="tree-node tables-section-node">
                        <button
                          class="tree-toggle"
                          aria-label="Toggle tables"
                          on:click={() => toggleTables(conn.id, db.name)}
                        >
                          <i
                            class="fas fa-chevron-{expandedTables[
                              `${conn.id}-${db.name}`
                            ]
                              ? 'down'
                              : 'right'}"
                          ></i>
                        </button>
                        <button
                          class="tree-section-header"
                          on:click={() => toggleTables(conn.id, db.name)}
                        >
                          <i class="fas fa-table"></i>
                          <span
                            >Tables ({expandedDatabases[`${conn.id}-${db.name}`]
                              .tables?.length || 0})</span
                          >
                        </button>
                      </div>
                      {#if expandedTables[`${conn.id}-${db.name}`]}
                        <div class="tree-children">
                          <table
                            class="table table-sm table-hover mb-0 table-borderless"
                            style="padding-left: 8px;"
                          >
                            <tbody>
                              {#each expandedDatabases[`${conn.id}-${db.name}`].tables || [] as table (table.name)}
                                <tr
                                  class="table-item-row"
                                  class:table-active={$selectedTable?.name ===
                                    table.name}
                                  style="cursor: pointer; line-height: 1.2;"
                                >
                                  <td
                                    class="p-0 align-middle"
                                    style="width: 100%; max-width: 0; overflow: hidden; white-space: nowrap; padding-left: 8px !important;"
                                  >
                                    <button
                                      class="btn btn-sm p-0 text-secondary"
                                      style="width: 20px; height: 20px; font-size: 10px; flex-shrink: 0;"
                                      aria-label="Toggle table"
                                    >
                                      <i class="fas fa-chevron-right"></i>
                                    </button>
                                    <button
                                      class="btn btn-sm p-1 text-start border-0"
                                      style="font-size: 12px; display: inline-block; max-width: calc(100% - 24px); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
                                      on:click={() => selectTable(table)}
                                      on:dblclick={() =>
                                        handleTableDoubleClick(table)}
                                    >
                                      <i
                                        class="fas fa-table text-secondary me-1"
                                        style="font-size: 11px;"
                                      ></i>
                                      <span class="text-truncate"
                                        >{table.name}</span
                                      >
                                    </button>
                                  </td>
                                  <td
                                    class="text-end align-middle"
                                    style="white-space: nowrap; width: 50px; min-width: 50px; max-width: 50px; padding: 2px 8px 2px 4px !important;"
                                  >
                                    {#if table.size_bytes !== undefined && table.size_bytes !== null && table.size_bytes > 0}
                                      <span
                                        class="badge bg-light text-secondary"
                                        style="font-size: 10px;"
                                        title="Table size"
                                        >{formatBytes(table.size_bytes)}</span
                                      >
                                    {/if}
                                  </td>
                                </tr>
                              {/each}
                            </tbody>
                          </table>
                        </div>
                      {/if}
                    </div>

                    <div class="tree-item">
                      <div class="tree-node tables-section-node">
                        <button class="tree-toggle" aria-label="Toggle views">
                          <i class="fas fa-chevron-right"></i>
                        </button>
                        <div class="tree-section-header">
                          <i class="fas fa-eye"></i>
                          <span>Views</span>
                        </div>
                      </div>
                    </div>

                    <div class="tree-item">
                      <div class="tree-node tables-section-node">
                        <button
                          class="tree-toggle"
                          aria-label="Toggle functions"
                        >
                          <i class="fas fa-chevron-right"></i>
                        </button>
                        <div class="tree-section-header">
                          <i class="fas fa-code"></i>
                          <span>Functions</span>
                        </div>
                      </div>
                    </div>
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </div>

  {#if storageInfo}
    <div class="border-top p-3 bg-body-secondary">
      <div
        class="d-flex align-items-center gap-2 mb-2 text-secondary text-uppercase"
        style="font-size: 11px; font-weight: 600; letter-spacing: 0.5px;"
      >
        <i class="fas fa-database"></i>
        <span>Storage</span>
      </div>
      <div class="d-flex flex-column gap-2">
        <div
          class="d-flex align-items-center gap-2 p-2 bg-body border rounded"
          style="font-size: 11px;"
          title={storageInfo.path}
        >
          <i class="fas fa-folder text-success" style="width: 14px;"></i>
          <span class="flex-grow-1 text-truncate"
            >{storageInfo.exists ? "Auto-saved" : "Not saved yet"}</span
          >
        </div>
        {#if storageInfo.exists}
          <div
            class="d-flex align-items-center gap-2 p-2 bg-body border rounded"
            style="font-size: 11px;"
          >
            <i class="fas fa-shield-alt text-success" style="width: 14px;"></i>
            <span class="flex-grow-1 text-truncate"
              >Encrypted ({(storageInfo.size_bytes / 1024).toFixed(1)} KB)</span
            >
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

{#if showModal}
  <ConnectionModal
    connection={editingConnection}
    on:close={closeModal}
    on:save={handleSaveConnection}
  />
{/if}

<style>
  /* Tree structure styles - keeping custom tree functionality */
  .tree-item {
    user-select: none;
  }

  .tree-node {
    display: flex;
    align-items: center;
    gap: 0px;
  }

  .tree-toggle {
    background: transparent;
    border: none;
    color: #6c757d;
    padding: 2px;
    cursor: pointer;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 10px;
    flex-shrink: 0;
    border-radius: 3px;
  }

  .tree-toggle:hover {
    background: #e9ecef;
    color: #212529;
  }

  .tree-label {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 4px;
    background: transparent;
    border: none;
    color: #212529;
    padding: 2px 8px;
    cursor: pointer;
    font-size: 12px;
    text-align: left;
    transition: background-color 0.15s;
    min-height: 20px;
    border-radius: 3px;
    line-height: 1.2;
  }

  .tree-label:hover {
    background: #e9ecef;
  }

  .tree-label.active {
    background: #cfe2ff;
    color: #0d6efd;
    font-weight: 500;
  }

  .tree-icon {
    font-size: 11px;
    width: 16px;
    text-align: center;
    flex-shrink: 0;
    color: #6c757d;
  }

  .tree-label.active .tree-icon {
    color: #0d6efd;
  }

  .tree-text {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tree-badge {
    font-size: 10px;
    padding: 2px 6px;
    background: #f8f9fa;
    border-radius: 3px;
    color: #6c757d;
    font-weight: 500;
    flex-shrink: 0;
    margin-left: auto;
  }

  .tree-label.active .tree-badge {
    background: #b6d4fe;
  }

  .tree-children {
    margin-left: 12px;
  }

  .tree-section-header {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    font-size: 11px;
    color: #6c757d;
    font-weight: 600;
    line-height: 1.2;
    flex: 1;
    background: transparent;
    border: none;
    text-align: left;
    cursor: pointer;
  }

  .tree-section-header:hover {
    background: #e9ecef;
  }

  .tree-section-header i {
    font-size: 11px;
    width: 16px;
    text-align: center;
  }

  .connection-node {
    padding-left: 8px;
  }

  .database-node {
    padding-left: 8px;
  }

  .tables-section-node {
    padding-left: 8px;
  }

  /* Custom styles for table item active state */
  .table-item-row.table-active .badge {
    background-color: #b6d4fe !important;
  }
</style>
