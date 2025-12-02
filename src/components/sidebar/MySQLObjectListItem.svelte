<script>
  import { createEventDispatcher } from "svelte";

  export let item;
  export let connection;
  export let database;
  export let schema = null;
  export let objectType = "view"; // view, index, procedure, trigger, event

  const dispatch = createEventDispatcher();

  function handleClick() {
    dispatch("select", { item, connection, database, schema, objectType });
  }

  function handleDoubleClick() {
    dispatch("doubleclick", { item, connection, database, schema, objectType });
  }

  function handleContextMenu(event) {
    dispatch("contextmenu", {
      event,
      item,
      connection,
      database,
      schema,
      objectType,
    });
  }

  // Get icon based on object type
  function getIcon() {
    switch (objectType) {
      case "view":
        return "fa-eye";
      case "index":
        return "fa-sort-alpha-down";
      case "procedure":
        return "fa-code";
      case "trigger":
        return "fa-bolt";
      case "event":
        return "fa-calendar-alt";
      default:
        return "fa-file";
    }
  }
</script>

<tr
  class="table-item-row"
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
      class="mysql-object-btn btn btn-sm p-1 text-start border-0"
      style="font-size: 12px; display: inline-block; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: middle;"
    >
      <i class="fas {getIcon()} mysql-object-icon me-1" style="font-size: 11px;"
      ></i>
      <span class="text-truncate" title={item.name}>{item.name}</span>
    </button>
  </td>
</tr>

<style>
  /* MySQL Object Items Styling (Views, Indexes, Procedures, Triggers, Events) */
  .table-item-row {
    background: transparent !important;
  }

  .table-item-row:hover {
    background: var(--hover-bg) !important;
  }

  .mysql-object-btn {
    color: var(--text-primary) !important;
    background: transparent !important;
  }

  .mysql-object-btn:hover {
    background: transparent !important;
  }

  .mysql-object-icon {
    color: var(--text-muted) !important;
  }

  .table-item-row:hover .mysql-object-icon {
    color: var(--text-primary) !important;
  }
</style>
