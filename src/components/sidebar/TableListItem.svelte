<script>
  import { createEventDispatcher } from "svelte";

  export let table;
  export let connection;
  export let database;
  export let isActive = false;
  export let dbType = "MySQL"; // MySQL, PostgreSQL, MongoDB, Redis, Ignite, MSSQL

  const dispatch = createEventDispatcher();

  function handleClick() {
    dispatch("select", { table, connection, database });
  }

  function handleDoubleClick() {
    dispatch("doubleclick", { table, connection, database });
  }

  function handleContextMenu(event) {
    dispatch("contextmenu", { event, table, connection, database });
  }

  // Get icon based on type
  function getIcon() {
    if (table.isView) return "fa-eye";
    if (dbType === "MongoDB") return "fa-layer-group";
    if (dbType === "Redis" || dbType === "Ignite") return "fa-server";
    return "fa-table";
  }

  function formatSize(bytes) {
    if (!bytes) return "";
    const kb = bytes / 1024;
    if (kb < 1024) return `${kb.toFixed(1)} KB`;
    const mb = kb / 1024;
    if (mb < 1024) return `${mb.toFixed(1)} MB`;
    const gb = mb / 1024;
    return `${gb.toFixed(1)} GB`;
  }
</script>

<tr
  class="table-item-row"
  class:table-active={isActive}
  style="cursor: pointer; line-height: 1.5;"
  on:click={handleClick}
  on:dblclick={handleDoubleClick}
  on:contextmenu={handleContextMenu}
>
  <td
    class="p-0 align-middle"
    style="width: 100%; max-width: 0; overflow: hidden; white-space: nowrap; padding-left: 8px !important;"
  >
    <button
      class="btn btn-sm p-0 text-secondary"
      style="width: 20px; height: 20px; font-size: 10px; flex-shrink: 0;"
      aria-label="Item"
    >
      <i class="fas fa-chevron-right"></i>
    </button>
    <button
      class="btn btn-sm p-1 text-start border-0"
      style="font-size: 12px; display: inline-block; max-width: calc(100% - 24px); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
    >
      <i class="fas {getIcon()} text-secondary me-1" style="font-size: 11px;"
      ></i>
      <span class="text-truncate" title={table.name}>{table.name}</span>
      {#if table.size}
        <span class="badge bg-secondary ms-1" style="font-size: 9px;">
          {formatSize(table.size)}
        </span>
      {/if}
    </button>
  </td>
</tr>

<style>
  /* Table/Collection/Cache item row styling - consistent with tree structure */
  .table-item-row {
    background: transparent !important;
  }

  .table-item-row:hover {
    background: var(--hover-bg) !important;
  }

  .table-item-row.table-active {
    background: var(--selected-bg) !important;
  }

  .table-item-row td {
    background: transparent !important;
  }

  /* Table/Collection/Cache item button styling - consistent with tree-label */
  .table-item-row .btn.text-start {
    color: var(--text-primary) !important;
    background: transparent !important;
  }

  .table-item-row .btn.text-start:hover {
    background: transparent !important;
  }

  .table-item-row.table-active .btn.text-start {
    color: var(--accent-blue) !important;
    font-weight: 500;
  }

  /* Icon color for table items - consistent with tree-icon */
  .table-item-row .fa-table,
  .table-item-row .fa-eye,
  .table-item-row .fa-server,
  .table-item-row .fa-layer-group {
    color: var(--text-muted) !important;
  }

  .table-item-row.table-active .fa-table,
  .table-item-row.table-active .fa-eye,
  .table-item-row.table-active .fa-server,
  .table-item-row.table-active .fa-layer-group {
    color: var(--accent-blue) !important;
  }

  /* Chevron button styling */
  .table-item-row .btn.text-secondary {
    color: var(--text-muted) !important;
    background: transparent !important;
  }

  /* Badge styling */
  .table-item-row.table-active .badge {
    background-color: var(--accent-blue-light) !important;
  }
</style>
