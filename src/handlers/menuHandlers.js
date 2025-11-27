import { fileService, showMessage, showError } from "../services/fileService";
import { get } from "svelte/store";
import { buildPaginatedQuery } from "../utils/defaultQueries";

/**
 * Handle opening table tab
 */
export async function handleOpenTableTab(
  event,
  tabStore,
  tabDataStore,
  getTableData
) {
  const { table, database, connection } = event.detail;

  tabStore.addTableTab(table, database, connection);

  const newTab = get(tabStore.activeTab);
  if (!newTab) return;

  try {
    let tableIdentifier = table.name;
    if (connection.db_type === "PostgreSQL" && table.schema) {
      tableIdentifier = `${table.schema}.${table.name}`;
    } else if (connection.db_type === "MySQL") {
      tableIdentifier = `${database.name}.${table.name}`;
    }

    const tableData = await getTableData(
      connection,
      database.name,
      tableIdentifier,
      200,
      0
    );

    // Build appropriate query based on database type
    let baseQuery;
    if (connection.db_type === "MongoDB") {
      // MongoDB uses JSON query format
      baseQuery = JSON.stringify({
        db: database.name,
        collection: table.name,
        operation: "find",
        query: {},
        options: { limit: 200 },
      });
    } else if (connection.db_type === "Redis") {
      // Redis uses command format
      baseQuery = `KEYS ${table.name}:*`;
    } else if (connection.db_type === "Ignite") {
      // Apache Ignite uses SCAN for cache data
      baseQuery = `SCAN ${database.name}`;
    } else if (connection.db_type === "PostgreSQL" && table.schema) {
      baseQuery = `SELECT * FROM "${table.schema}"."${table.name}"`;
    } else if (connection.db_type === "MySQL") {
      baseQuery = `SELECT * FROM ${database.name}.${table.name}`;
    } else if (connection.db_type === "MSSQL") {
      // SQL Server uses [database].[schema].[table] format
      baseQuery = `SELECT * FROM [${database.name}].[dbo].[${table.name}]`;
    } else {
      baseQuery = `SELECT * FROM ${table.name}`;
    }

    const tableQuery = buildPaginatedQuery(
      connection.db_type,
      baseQuery,
      200,
      0
    );

    tabDataStore.setQueryResult(newTab.id, tableData);
    tabDataStore.setExecutedQuery(newTab.id, tableQuery);
  } catch (error) {
    console.error("Failed to load table data:", error);
    await showError(`Failed to load table data: ${error.message || error}`);
  }
}

/**
 * Menu Action Handlers
 */
export function createMenuHandlers(context) {
  const {
    tabs,
    activeTab,
    tabDataStore,
    addNewQueryTab,
    activeConnection,
    showModal,
    showSidebar,
    showToolbar,
    showAboutModal,
    runningQueries,
    getTableData,
  } = context;

  return {
    // File Menu Handlers
    async handleOpenFile() {
      try {
        const file = await fileService.openFile();

        if (file) {
          const newTab = {
            id: Date.now(),
            title: file.name || `Query ${tabs.length + 1}`,
            type: "query",
            modified: false,
            filePath: file.path,
          };
          tabs.push(newTab);
          context.setActiveTab(newTab);
          tabDataStore.setQueryText(newTab.id, file.content);
        }
      } catch (error) {
        console.error("Failed to open file:", error);
        await showError("Failed to open file: " + error.message);
      }
    },

    async handleSaveQuery() {
      if (!activeTab || activeTab.type !== "query") {
        await showError("No query tab is active");
        return;
      }

      const tabData = tabDataStore.get(activeTab.id);
      if (!tabData || !tabData.queryText) {
        await showError("No query to save");
        return;
      }

      try {
        if (activeTab.filePath) {
          await fileService.saveQuery(activeTab.filePath, tabData.queryText);
          activeTab.modified = false;
          context.updateTabs();
          await showMessage("Query saved successfully");
        } else {
          await this.handleSaveAs();
        }
      } catch (error) {
        console.error("Failed to save query:", error);
        await showError("Failed to save query: " + error.message);
      }
    },

    async handleSaveAs() {
      if (!activeTab || activeTab.type !== "query") {
        await showError("No query tab is active");
        return;
      }

      const tabData = tabDataStore.get(activeTab.id);
      if (!tabData || !tabData.queryText) {
        await showError("No query to save");
        return;
      }

      try {
        const result = await fileService.saveQueryAs(
          activeTab.title,
          tabData.queryText
        );

        if (result) {
          activeTab.filePath = result.path;
          activeTab.modified = false;
          activeTab.title = result.name;
          context.updateTabs();
          await showMessage("Query saved successfully");
        }
      } catch (error) {
        console.error("Failed to save query:", error);
        await showError("Failed to save query: " + error.message);
      }
    },

    async handleExportData() {
      if (!activeTab) {
        await showError("No active tab");
        return;
      }

      const tabData = tabDataStore.get(activeTab.id);
      if (!tabData || !tabData.queryResult) {
        await showError("No data to export");
        return;
      }

      try {
        const success = await fileService.exportData(tabData.queryResult);
        if (success) {
          await showMessage("Data exported successfully");
        }
      } catch (error) {
        console.error("Failed to export data:", error);
        await showError("Failed to export data: " + error.message);
      }
    },

    async handleImportData() {
      try {
        const file = await fileService.importData();
        if (file) {
          await showMessage("Import functionality will be available soon");
        }
      } catch (error) {
        console.error("Failed to import data:", error);
        await showError("Failed to import data: " + error.message);
      }
    },

    // Edit Menu Handlers
    handleUndo() {
      if (activeTab && activeTab.type === "query") {
        document.dispatchEvent(
          new CustomEvent("editor-undo", { detail: { tabId: activeTab.id } })
        );
      }
    },

    handleRedo() {
      if (activeTab && activeTab.type === "query") {
        document.dispatchEvent(
          new CustomEvent("editor-redo", { detail: { tabId: activeTab.id } })
        );
      }
    },

    async handleCopy() {
      try {
        const selectedText = window.getSelection().toString();
        if (selectedText) {
          await navigator.clipboard.writeText(selectedText);
        }
      } catch (error) {
        console.error("Failed to copy:", error);
      }
    },

    async handlePaste() {
      if (activeTab && activeTab.type === "query") {
        try {
          const text = await navigator.clipboard.readText();
          document.dispatchEvent(
            new CustomEvent("editor-paste", {
              detail: { tabId: activeTab.id, text },
            })
          );
        } catch (error) {
          console.error("Failed to paste:", error);
        }
      }
    },

    // View Menu Handlers
    handleToggleToolbar() {
      context.setShowToolbar(!showToolbar);
    },

    async handleViewColumns() {
      if (!activeTab) {
        await showError("No active tab");
        return;
      }

      const tabData = tabDataStore.get(activeTab.id);
      if (!tabData || !tabData.queryResult) {
        await showError("No data available");
        return;
      }

      const columns = tabData.queryResult.columns || [];
      const columnInfo = columns
        .map((col) => `${col.name} (${col.data_type || "unknown"})`)
        .join("\n");
      await showMessage(`Columns:\n\n${columnInfo}`, "Table Columns");
    },

    // Database Menu Handlers
    async handleConnect() {
      if (!activeConnection) {
        await showError(
          "No connection selected. Please create a connection first."
        );
        context.setShowModal(true);
      } else {
        await showMessage("Already connected to: " + activeConnection.name);
      }
    },

    async handleDisconnect() {
      if (!activeConnection) {
        await showError("No active connection");
        return;
      }

      try {
        const { disconnectFromDatabase } = await import("../utils/tauri");
        const connectionId = activeConnection.id;
        await disconnectFromDatabase(connectionId);

        // Close all tabs for this connection and get closed tab IDs
        const closedTabIds =
          context.tabStore.closeTabsByConnection(connectionId);

        // Clean up tab data for closed tabs
        if (closedTabIds.length > 0) {
          tabDataStore.removeTabsByIds(closedTabIds);
        }

        await showMessage("Disconnected successfully");
      } catch (error) {
        console.error("Failed to disconnect:", error);
        await showError("Failed to disconnect: " + error.message);
      }
    },

    // Toolbar Handlers
    async handleExecute() {
      if (!activeTab || activeTab.type !== "query") {
        await showError("No query tab is active");
        return;
      }

      document.dispatchEvent(
        new CustomEvent("execute-query", {
          detail: { tabId: activeTab.id },
        })
      );
    },

    async handleExecuteScript() {
      if (!activeTab || activeTab.type !== "query") {
        await showError("No query tab is active");
        return;
      }

      document.dispatchEvent(
        new CustomEvent("execute-script", {
          detail: { tabId: activeTab.id },
        })
      );
    },

    handleStop() {
      if (!activeTab) return;

      const controller = runningQueries.get(activeTab.id);
      if (controller) {
        controller.abort();
        runningQueries.delete(activeTab.id);

        document.dispatchEvent(
          new CustomEvent("stop-query", {
            detail: { tabId: activeTab.id },
          })
        );
      }
    },

    async handleCommit() {
      if (!activeConnection) {
        await showError("No active connection");
        return;
      }

      try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("execute_query", {
          connection: activeConnection,
          query: "COMMIT",
        });
        await showMessage("Transaction committed successfully");
      } catch (error) {
        console.error("Failed to commit transaction:", error);
        await showError("Failed to commit transaction: " + error.message);
      }
    },

    async handleRollback() {
      if (!activeConnection) {
        await showError("No active connection");
        return;
      }

      try {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("execute_query", {
          connection: activeConnection,
          query: "ROLLBACK",
        });
        await showMessage("Transaction rolled back successfully");
      } catch (error) {
        console.error("Failed to rollback transaction:", error);
        await showError("Failed to rollback transaction: " + error.message);
      }
    },

    async handleRefresh() {
      if (!activeTab) {
        await showError("No active tab");
        return;
      }

      if (activeTab.type === "table") {
        try {
          const tableInfo = activeTab.tableInfo;
          let tableIdentifier = tableInfo.name;

          if (
            tableInfo.connection.db_type === "PostgreSQL" &&
            tableInfo.schema
          ) {
            tableIdentifier = `${tableInfo.schema}.${tableInfo.name}`;
          } else if (tableInfo.connection.db_type === "MySQL") {
            tableIdentifier = `${tableInfo.database}.${tableInfo.name}`;
          }

          const tableData = await getTableData(
            tableInfo.connection,
            tableInfo.database,
            tableIdentifier,
            200,
            0
          );

          tabDataStore.setQueryResult(activeTab.id, tableData);
          await showMessage("Table data refreshed");
        } catch (error) {
          console.error("Failed to refresh table data:", error);
          await showError("Failed to refresh: " + error.message);
        }
      } else if (activeTab.type === "query") {
        const tabData = tabDataStore.get(activeTab.id);
        if (tabData && tabData.executedQuery) {
          document.dispatchEvent(
            new CustomEvent("execute-query", {
              detail: { tabId: activeTab.id },
            })
          );
        } else {
          await showError("No executed query to refresh");
        }
      }
    },

    // Help Menu Handlers
    async handleDocumentation() {
      try {
        const { open } = await import("@tauri-apps/plugin-shell");
        await open("https://github.com/yourusername/rustdbgrid#readme");
      } catch (error) {
        console.error("Failed to open documentation:", error);
        await showMessage(
          "Documentation: https://github.com/yourusername/rustdbgrid"
        );
      }
    },

    handleAbout() {
      context.setShowAboutModal(true);
    },
  };
}
