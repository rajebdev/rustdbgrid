<script>
  import BaseContextMenu from "../../../shared/components/base/BaseContextMenu.svelte";
  import { createEventDispatcher } from "svelte";

  export let x = 0;
  export let y = 0;
  export let database = null;
  export let connection = null;

  const dispatch = createEventDispatcher();

  const menuItems = [
    {
      type: "item",
      id: "sql-editor",
      label: "SQL Editor",
      icon: "fas fa-terminal",
      shortcut: "F4",
      action: "sqlEditor",
    },
    {
      type: "item",
      id: "create",
      label: "Create (TODO)",
      icon: "fas fa-plus",
      disabled: true,
      action: "create",
    },
    { type: "divider" },
    {
      type: "item",
      id: "view-database",
      label: "View Database",
      icon: "fas fa-eye",
      shortcut: "F4",
      action: "viewDatabase",
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
      id: "compare-migrate",
      label: "Compare/Migrate (TODO)",
      icon: "fas fa-code-compare",
      disabled: true,
      action: "compareMigrate",
    },
    {
      type: "item",
      id: "tools",
      label: "Tools (TODO)",
      icon: "fas fa-wrench",
      disabled: true,
      action: "tools",
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
      icon: "fas fa-sync",
      shortcut: "F5",
      action: "refresh",
    },
  ];

  function handleAction(event) {
    const { action } = event.detail;
    dispatch(action, { database, connection });
  }
</script>

<BaseContextMenu {x} {y} items={menuItems} on:action={handleAction} on:close />
