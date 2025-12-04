<script>
  import BaseContextMenu from "./base/BaseContextMenu.svelte";
  import { createEventDispatcher } from "svelte";

  export let x = 0;
  export let y = 0;
  export let view = null;
  export let connection = null;
  export let database = null;

  const dispatch = createEventDispatcher();

  const menuItems = [
    {
      type: "item",
      id: "view-structure",
      label: "View Structure",
      icon: "fas fa-eye",
      shortcut: "F4",
      action: "viewStructure",
    },
    {
      type: "item",
      id: "view-definition",
      label: "View Definition",
      icon: "fas fa-code",
      action: "viewDefinition",
    },
    {
      type: "item",
      id: "view-data",
      label: "View Data",
      icon: "fas fa-table",
      action: "viewData",
    },
    { type: "divider" },
    {
      type: "item",
      id: "export-data",
      label: "Export Data",
      icon: "fas fa-file-export",
      action: "exportData",
    },
    {
      type: "item",
      id: "import-data",
      label: "Import Data",
      icon: "fas fa-file-import",
      action: "importData",
    },
    { type: "divider" },
    {
      type: "item",
      id: "read-console",
      label: "Read in Console",
      icon: "fas fa-terminal",
      action: "readInConsole",
    },
    { type: "divider" },
    {
      type: "item",
      id: "copy",
      label: "Copy",
      icon: "fas fa-copy",
      action: "copy",
    },
    {
      type: "item",
      id: "copy-advanced",
      label: "Copy Advanced Info",
      icon: "fas fa-info-circle",
      action: "copyAdvancedInfo",
    },
    { type: "divider" },
    {
      type: "item",
      id: "delete",
      label: "Delete",
      icon: "fas fa-trash",
      danger: true,
      action: "delete",
    },
    {
      type: "item",
      id: "rename",
      label: "Rename",
      icon: "fas fa-edit",
      action: "rename",
    },
    { type: "divider" },
    {
      type: "item",
      id: "refresh",
      label: "Refresh",
      icon: "fas fa-sync-alt",
      action: "refresh",
    },
  ];

  function handleAction(event) {
    const { action } = event.detail;
    dispatch(action, { view, connection, database });
  }
</script>

<BaseContextMenu {x} {y} items={menuItems} on:action={handleAction} on:close />

<!-- View Structure -->
<button
  class="dropdown-item d-flex align-items-center justify-content-between"
  on:click={() => handleAction("viewStructure")}
>
  <span>
    <i class="fas fa-eye me-2"></i>
    View Structure
  </span>
  <span class="text-muted ms-3 small">F4</span>
</button>

<!-- View Definition -->
<button class="dropdown-item" on:click={() => handleAction("viewDefinition")}>
  <i class="fas fa-code me-2"></i>
  View Definition
</button>

<!-- View Data -->
<button class="dropdown-item" on:click={() => handleAction("viewData")}>
  <i class="fas fa-table me-2"></i>
  View Data
</button>

<div class="dropdown-divider"></div>

<!-- Export Data -->
<button class="dropdown-item" on:click={() => handleAction("exportData")}>
  <i class="fas fa-file-export me-2"></i>
  Export Data
</button>

<!-- Import Data -->
<button class="dropdown-item" on:click={() => handleAction("importData")}>
  <i class="fas fa-file-import me-2"></i>
  Import Data
</button>

<div class="dropdown-divider"></div>

<!-- Read in Console -->
<button class="dropdown-item" on:click={() => handleAction("readInConsole")}>
  <i class="fas fa-terminal me-2"></i>
  Read in Console
</button>

<div class="dropdown-divider"></div>

<!-- Copy -->
<button class="dropdown-item" on:click={() => handleAction("copy")}>
  <i class="fas fa-copy me-2"></i>
  Copy
</button>

<!-- Copy Advanced Info -->
<button class="dropdown-item" on:click={() => handleAction("copyAdvancedInfo")}>
  <i class="fas fa-info-circle me-2"></i>
  Copy Advanced Info
</button>

<div class="dropdown-divider"></div>

<!-- Rename -->
<button class="dropdown-item" on:click={() => handleAction("rename")}>
  <i class="fas fa-edit me-2"></i>
  Rename
</button>

<!-- Delete -->
<button
  class="dropdown-item text-danger"
  on:click={() => handleAction("delete")}
>
  <i class="fas fa-trash me-2"></i>
  Delete
</button>

<div class="dropdown-divider"></div>

<!-- Refresh -->
<button class="dropdown-item" on:click={() => handleAction("refresh")}>
  <i class="fas fa-sync-alt me-2"></i>
  Refresh
</button>
