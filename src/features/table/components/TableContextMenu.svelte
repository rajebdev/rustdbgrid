<script>
  import BaseContextMenu from "./base/BaseContextMenu.svelte";
  import { createEventDispatcher } from "svelte";

  export let x = 0;
  export let y = 0;
  export let table = null;
  export let connection = null;
  export let database = null;

  const dispatch = createEventDispatcher();

  const menuItems = [
    {
      type: "item",
      id: "view-table",
      label: "View Table",
      icon: "fas fa-table",
      shortcut: "F4",
      action: "viewTable",
    },
    {
      type: "item",
      id: "filter",
      label: "Filter (TODO)",
      icon: "fas fa-filter",
      disabled: true,
      action: "filter",
    },
    {
      type: "item",
      id: "view-diagram",
      label: "View Diagram",
      icon: "fas fa-project-diagram",
      shortcut: "Ctrl+Shift+Enter",
      action: "viewDiagram",
    },
    { type: "divider" },
    {
      type: "item",
      id: "view-data",
      label: "View Data",
      icon: "fas fa-eye",
      action: "viewData",
    },
    { type: "divider" },
    {
      type: "item",
      id: "compare-migrate",
      label: "Compare/Migrate (TODO)",
      icon: "fas fa-exchange-alt",
      disabled: true,
      action: "compareMigrate",
    },
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
      id: "tools",
      label: "Tools (TODO)",
      icon: "fas fa-tools",
      disabled: true,
      action: "tools",
    },
    {
      type: "item",
      id: "generate-sql",
      label: "Generate SQL (TODO)",
      icon: "fas fa-code",
      disabled: true,
      action: "generateSql",
    },
    {
      type: "item",
      id: "read-console",
      label: "Read data in SQL console",
      icon: "fas fa-terminal",
      action: "readInConsole",
    },
    { type: "divider" },
    {
      type: "item",
      id: "copy",
      label: "Copy",
      icon: "fas fa-copy",
      shortcut: "Ctrl+C",
      action: "copy",
    },
    {
      type: "item",
      id: "paste",
      label: "Paste",
      icon: "fas fa-paste",
      shortcut: "Ctrl+V",
      action: "paste",
    },
    {
      type: "item",
      id: "copy-advanced",
      label: "Copy Advanced Info",
      icon: "fas fa-info-circle",
      shortcut: "Ctrl+Shift+C",
      action: "copyAdvancedInfo",
    },
    { type: "divider" },
    {
      type: "item",
      id: "delete",
      label: "Delete",
      icon: "fas fa-trash",
      shortcut: "Delete",
      danger: true,
      action: "delete",
    },
    {
      type: "item",
      id: "rename",
      label: "Rename",
      icon: "fas fa-edit",
      shortcut: "F2",
      action: "rename",
    },
    { type: "divider" },
    {
      type: "item",
      id: "refresh",
      label: "Refresh",
      icon: "fas fa-sync-alt",
      shortcut: "F5",
      action: "refresh",
    },
  ];

  function handleAction(event) {
    const { action } = event.detail;
    dispatch(action, { table, connection, database });
  }
</script>

<BaseContextMenu {x} {y} items={menuItems} on:action={handleAction} on:close />
