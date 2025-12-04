<script>
  import { onMount, onDestroy } from "svelte";
  import { sidebarStore } from "../../stores/sidebar";

  import ConnectionContextMenu from "../../../connection/components/ConnectionContextMenu.svelte";
  import DatabaseContextMenu from "../../../database/components/DatabaseContextMenu.svelte";
  import SchemaContextMenu from "../../../database/components/SchemaContextMenu.svelte";
  import TableContextMenu from "../../../table/components/TableContextMenu.svelte";
  import ViewContextMenu from "../../../database/components/ViewContextMenu.svelte";

  export let connectedConnections = {};
  export let onConnectionEdit = null;
  export let onConnectionDelete = null;
  export let onConnectionRefresh = null;
  export let onConnectionConnect = null;
  export let onConnectionDisconnect = null;
  export let onConnectionCopy = null;
  export let onConnectionRename = null;
  export let onDatabaseSqlEditor = null;
  export let onDatabaseView = null;
  export let onDatabaseCopy = null;
  export let onDatabasePaste = null;
  export let onDatabaseCopyAdvancedInfo = null;
  export let onDatabaseDelete = null;
  export let onDatabaseRename = null;
  export let onDatabaseRefresh = null;
  export let onSchemaSqlEditor = null;
  export let onSchemaView = null;
  export let onSchemaViewDiagram = null;
  export let onSchemaImportData = null;
  export let onSchemaGenerateSql = null;
  export let onSchemaCopy = null;
  export let onSchemaPaste = null;
  export let onSchemaCopyAdvancedInfo = null;
  export let onSchemaDelete = null;
  export let onSchemaRename = null;
  export let onSchemaRefresh = null;
  export let onTableViewTable = null;
  export let onTableViewDiagram = null;
  export let onTableViewData = null;
  export let onTableExportData = null;
  export let onTableImportData = null;
  export let onTableReadInConsole = null;
  export let onTableCopy = null;
  export let onTablePaste = null;
  export let onTableCopyAdvancedInfo = null;
  export let onTableDelete = null;
  export let onTableRename = null;
  export let onTableRefresh = null;
  export let onViewStructure = null;
  export let onViewDefinition = null;
  export let onViewData = null;
  export let onViewExportData = null;
  export let onViewImportData = null;
  export let onViewReadInConsole = null;
  export let onViewCopy = null;
  export let onViewCopyAdvancedInfo = null;
  export let onViewRename = null;
  export let onViewDelete = null;
  export let onViewRefresh = null;

  let contextMenu = null;

  const unsubscribe = sidebarStore.subscribe((state) => {
    contextMenu = state.contextMenu;
  });

  onMount(() => {
    const handleClick = () => {
      sidebarStore.closeContextMenu();
    };

    document.addEventListener("click", handleClick);

    return () => {
      document.removeEventListener("click", handleClick);
    };
  });

  onDestroy(() => {
    unsubscribe();
  });

  function closeMenu() {
    sidebarStore.closeContextMenu();
  }
</script>

{#if contextMenu}
  {#if contextMenu.type === "connection"}
    <ConnectionContextMenu
      x={contextMenu.x}
      y={contextMenu.y}
      connection={contextMenu.connection}
      isConnected={connectedConnections[contextMenu.connection.id]}
      on:edit={(e) => {
        onConnectionEdit?.(e);
        closeMenu();
      }}
      on:delete={(e) => {
        onConnectionDelete?.(e);
        closeMenu();
      }}
      on:refresh={(e) => {
        onConnectionRefresh?.(e);
        closeMenu();
      }}
      on:connect={(e) => {
        onConnectionConnect?.(e);
        closeMenu();
      }}
      on:disconnect={(e) => {
        onConnectionDisconnect?.(e);
        closeMenu();
      }}
      on:copy={(e) => {
        onConnectionCopy?.(e);
        closeMenu();
      }}
      on:rename={(e) => {
        onConnectionRename?.(e);
        closeMenu();
      }}
    />
  {:else if contextMenu.type === "database"}
    <DatabaseContextMenu
      x={contextMenu.x}
      y={contextMenu.y}
      database={contextMenu.database}
      connection={contextMenu.connection}
      on:sqlEditor={(e) => {
        onDatabaseSqlEditor?.(e);
        closeMenu();
      }}
      on:viewDatabase={(e) => {
        onDatabaseView?.(e);
        closeMenu();
      }}
      on:copy={(e) => {
        onDatabaseCopy?.(e);
        closeMenu();
      }}
      on:paste={(e) => {
        onDatabasePaste?.(e);
        closeMenu();
      }}
      on:copyAdvancedInfo={(e) => {
        onDatabaseCopyAdvancedInfo?.(e);
        closeMenu();
      }}
      on:delete={(e) => {
        onDatabaseDelete?.(e);
        closeMenu();
      }}
      on:rename={(e) => {
        onDatabaseRename?.(e);
        closeMenu();
      }}
      on:refresh={(e) => {
        onDatabaseRefresh?.(e);
        closeMenu();
      }}
    />
  {:else if contextMenu.type === "schema"}
    <SchemaContextMenu
      x={contextMenu.x}
      y={contextMenu.y}
      schema={contextMenu.schema}
      database={contextMenu.database}
      connection={contextMenu.connection}
      on:sqlEditor={(e) => {
        onSchemaSqlEditor?.(e);
        closeMenu();
      }}
      on:viewSchema={(e) => {
        onSchemaView?.(e);
        closeMenu();
      }}
      on:viewDiagram={(e) => {
        onSchemaViewDiagram?.(e);
        closeMenu();
      }}
      on:importData={(e) => {
        onSchemaImportData?.(e);
        closeMenu();
      }}
      on:generateSql={(e) => {
        onSchemaGenerateSql?.(e);
        closeMenu();
      }}
      on:copy={(e) => {
        onSchemaCopy?.(e);
        closeMenu();
      }}
      on:paste={(e) => {
        onSchemaPaste?.(e);
        closeMenu();
      }}
      on:copyAdvancedInfo={(e) => {
        onSchemaCopyAdvancedInfo?.(e);
        closeMenu();
      }}
      on:delete={(e) => {
        onSchemaDelete?.(e);
        closeMenu();
      }}
      on:rename={(e) => {
        onSchemaRename?.(e);
        closeMenu();
      }}
      on:refresh={(e) => {
        onSchemaRefresh?.(e);
        closeMenu();
      }}
    />
  {:else if contextMenu.type === "table"}
    <TableContextMenu
      x={contextMenu.x}
      y={contextMenu.y}
      table={contextMenu.table}
      database={contextMenu.database}
      connection={contextMenu.connection}
      on:viewTable={(e) => {
        onTableViewTable?.(e);
        closeMenu();
      }}
      on:viewDiagram={(e) => {
        onTableViewDiagram?.(e);
        closeMenu();
      }}
      on:viewData={(e) => {
        onTableViewData?.(e);
        closeMenu();
      }}
      on:exportData={(e) => {
        onTableExportData?.(e);
        closeMenu();
      }}
      on:importData={(e) => {
        onTableImportData?.(e);
        closeMenu();
      }}
      on:readInConsole={(e) => {
        onTableReadInConsole?.(e);
        closeMenu();
      }}
      on:copy={(e) => {
        onTableCopy?.(e);
        closeMenu();
      }}
      on:paste={(e) => {
        onTablePaste?.(e);
        closeMenu();
      }}
      on:copyAdvancedInfo={(e) => {
        onTableCopyAdvancedInfo?.(e);
        closeMenu();
      }}
      on:delete={(e) => {
        onTableDelete?.(e);
        closeMenu();
      }}
      on:rename={(e) => {
        onTableRename?.(e);
        closeMenu();
      }}
      on:refresh={(e) => {
        onTableRefresh?.(e);
        closeMenu();
      }}
    />
  {:else if contextMenu.type === "view"}
    <ViewContextMenu
      x={contextMenu.x}
      y={contextMenu.y}
      view={contextMenu.view}
      database={contextMenu.database}
      connection={contextMenu.connection}
      on:viewStructure={(e) => {
        onViewStructure?.(e);
        closeMenu();
      }}
      on:viewDefinition={(e) => {
        onViewDefinition?.(e);
        closeMenu();
      }}
      on:viewData={(e) => {
        onViewData?.(e);
        closeMenu();
      }}
      on:exportData={(e) => {
        onViewExportData?.(e);
        closeMenu();
      }}
      on:importData={(e) => {
        onViewImportData?.(e);
        closeMenu();
      }}
      on:readInConsole={(e) => {
        onViewReadInConsole?.(e);
        closeMenu();
      }}
      on:copy={(e) => {
        onViewCopy?.(e);
        closeMenu();
      }}
      on:copyAdvancedInfo={(e) => {
        onViewCopyAdvancedInfo?.(e);
        closeMenu();
      }}
      on:rename={(e) => {
        onViewRename?.(e);
        closeMenu();
      }}
      on:delete={(e) => {
        onViewDelete?.(e);
        closeMenu();
      }}
      on:refresh={(e) => {
        onViewRefresh?.(e);
        closeMenu();
      }}
    />
  {/if}
{/if}
