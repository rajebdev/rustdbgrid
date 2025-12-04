<script>
  import BaseContextMenu from "../../../shared/components/base/BaseContextMenu.svelte";
  import { createEventDispatcher } from "svelte";

  export let x = 0;
  export let y = 0;
  export let connection = null;
  export let isConnected = false;

  const dispatch = createEventDispatcher();

  $: menuItems = [
    {
      type: "item",
      id: "sql-editor",
      label: "SQL Editor (TODO)",
      icon: "fas fa-code",
      shortcut: "F3",
      disabled: true,
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
      id: "edit",
      label: "Edit Connection",
      icon: "fas fa-edit",
      shortcut: "F4",
      action: "edit",
    },
    {
      type: "item",
      id: "connection-view",
      label: "Connection view (TODO)",
      icon: "fas fa-eye",
      disabled: true,
      action: "connectionView",
    },
    {
      type: "item",
      id: "browse",
      label: "Browse from here (TODO)",
      icon: "fas fa-folder-open",
      disabled: true,
      action: "browse",
    },
    { type: "divider" },
    {
      type: "item",
      id: "connect",
      label: "Connect",
      icon: "fas fa-plug",
      disabled: isConnected,
      action: "connect",
    },
    {
      type: "item",
      id: "invalidate",
      label: "Invalidate/Reconnect",
      icon: "fas fa-sync",
      disabled: !isConnected,
      action: "refresh",
    },
    {
      type: "item",
      id: "disconnect",
      label: "Disconnect",
      icon: "fas fa-unlink",
      disabled: !isConnected,
      action: "disconnect",
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
      id: "tools",
      label: "Tools (TODO)",
      icon: "fas fa-tools",
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
      label: "Paste (TODO)",
      icon: "fas fa-paste",
      shortcut: "Ctrl+V",
      disabled: true,
      action: "paste",
    },
    {
      type: "item",
      id: "copy-advanced",
      label: "Copy Advanced Info (TODO)",
      icon: "fas fa-clone",
      shortcut: "Ctrl+Shift+C",
      disabled: true,
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
      icon: "fas fa-pen",
      shortcut: "F2",
      action: "rename",
    },
    { type: "divider" },
    {
      type: "item",
      id: "refresh",
      label: "Refresh",
      icon: "fas fa-redo",
      shortcut: "F5",
      action: "refresh",
    },
  ];

  function handleAction(event) {
    const { action } = event.detail;
    dispatch(action, connection);
  }
</script>

<BaseContextMenu {x} {y} items={menuItems} on:action={handleAction} on:close />
