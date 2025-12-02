import { fileService, showMessage, showError } from "../services/fileService";
import { get } from "svelte/store";
import {
  executeQuery,
  getNextQueryNumber,
  loadTableData,
} from "../utils/tauri";
import { recentFilesStore } from "../stores/recentFiles";

/**
 * Handle opening table tab
 */
export async function handleOpenTableTab(event, tabStore, tabDataStore) {
  const { table, database, connection } = event.detail;

  tabStore.addTableTab(table, database, connection);

  const newTab = get(tabStore.activeTab);
  if (!newTab) return;

  try {
    // Use new loadTableData API
    const tableData = await loadTableData(
      connection.id,
      connection.db_type,
      table.name,
      {
        database: database.name,
        schema: table.schema || null,
        limit: 200,
        offset: 0,
        filters: [],
        orderBy: [],
      }
    );

    tabDataStore.setQueryResult(newTab.id, tableData);

    // Store the final query for reference
    if (tableData.final_query) {
      tabDataStore.setExecutedQuery(newTab.id, tableData.final_query);
    }
  } catch (error) {
    console.error("Failed to load table data:", error);
    await showError(`Failed to load table data: ${error.message || error}`);
  }
}

/**
 * Handle opening procedure tab (shows procedure source code)
 */
export async function handleOpenProcedureTab(event, tabStore, tabDataStore) {
  const { procedure, database, connection } = event.detail;

  // Add a procedure tab
  tabStore.addProcedureTab(procedure, database, connection);
}

/**
 * Menu Action Handlers
 */
export function createMenuHandlers(context) {
  const {
    tabStore,
    tabDataStore,
    addNewQueryTab,
    activeConnection,
    showModal,
    showSidebar,
    showToolbar,
    showAboutModal,
    runningQueries,
    updateTabs,
  } = context;

  // Get activeTab from tabStore (it's a derived store)
  const activeTab = tabStore.activeTab;

  return {
    // File Menu Handlers
    async handleNewQuery() {
      try {
        // Get next query number from both files and active tabs
        const currentTabs = get(tabStore);
        const queryTabs = currentTabs.filter(
          (t) => t.type === "query" && t.title?.startsWith("Query ")
        );

        // Extract numbers from active tabs
        let maxTabNumber = 0;
        queryTabs.forEach((tab) => {
          const match = tab.title.match(/^Query (\d+)/);
          if (match) {
            const num = parseInt(match[1], 10);
            if (num > maxTabNumber) maxTabNumber = num;
          }
        });

        // Get next query number from files
        const fileQueryNumber = await getNextQueryNumber();

        // Use the higher of the two
        const queryNumber = Math.max(maxTabNumber + 1, fileQueryNumber);
        const fileName = `Query ${queryNumber}.sql`;

        // New query should always start with a template
        const newContent = "-- SQL Query\nSELECT * FROM table_name LIMIT 100;";

        // Add new tab with unique filename and initial content
        // addQueryTab will automatically initialize tabDataStore with the content
        tabStore.addQueryTab(get(activeConnection), newContent);

        // Get the newly created tab and update its title and file path
        const newTab = get(activeTab);
        if (newTab) {
          // Update tab properties
          newTab.title = fileName;
          newTab.filePath = `queries/${fileName}`;

          // Save the updated tab back to the store
          tabStore.updateTab(newTab);

          // Auto-save the new query to local file immediately
          try {
            const { invoke } = await import("@tauri-apps/api/core");
            const configDir = await invoke("get_config_dir");
            const sep = navigator.platform.toLowerCase().includes("win")
              ? "\\"
              : "/";
            const fullPath = `${configDir}${sep}rustdbgrid${sep}queries${sep}${fileName}`;

            await invoke("auto_save_query_file", {
              filePath: fullPath,
              content: newContent,
            });
          } catch (saveError) {
            console.error("Failed to auto-save new query file:", saveError);
          }
        }
      } catch (error) {
        console.error("Failed to create new query:", error);
        await showError("Failed to create new query: " + error.message);
      }
    },

    async handleOpenFile() {
      try {
        const file = await fileService.openFile();

        if (file) {
          // Add to recent files
          recentFilesStore.addFile(file.path, file.name);

          // Create new query tab
          tabStore.addQueryTab(get(activeConnection), file.content);

          // Get the newly created tab
          const newTab = get(tabStore.activeTab);
          if (newTab) {
            // Update with file info
            newTab.filePath = file.path;
            newTab.title = file.name.replace(/\.[^/.]+$/, ""); // Remove extension
            context.updateTabs();
          }
        }
      } catch (error) {
        console.error("Failed to open file:", error);
        await showError("Failed to open file: " + error.message);
      }
    },

    async handleOpenRecentFile(file) {
      try {
        // Read file content
        const { readTextFile } = await import("@tauri-apps/plugin-fs");
        const content = await readTextFile(file.path);

        // Update recent files (move to top)
        recentFilesStore.addFile(file.path, file.name);

        // Create new query tab
        tabStore.addQueryTab(get(activeConnection), content);

        // Get the newly created tab
        const newTab = get(tabStore.activeTab);
        if (newTab) {
          // Update with file info
          newTab.filePath = file.path;
          newTab.title = file.name.replace(/\.[^/.]+$/, ""); // Remove extension
          context.updateTabs();
        }
      } catch (error) {
        console.error("Failed to open recent file:", error);
        await showError("Failed to open file: " + error.message);
        // Remove from recent files if it doesn't exist
        recentFilesStore.removeFile(file.path);
      }
    },

    async handleSaveQuery() {
      const currentTab = get(activeTab);
      if (!currentTab || currentTab.type !== "query") {
        await showError("No query tab is active");
        return;
      }

      const tabData = tabDataStore.get(currentTab.id);
      if (!tabData || !tabData.queryText) {
        await showError("No query to save");
        return;
      }

      try {
        // Import saveStatus store
        const { isSaving, saveStatus } = await import("../stores/connections");

        // Set saving status
        isSaving.set(true);
        saveStatus.set({
          message: "Saving...",
          type: "info",
          timestamp: Date.now(),
        });

        // Auto-save to queries folder
        const fileName = currentTab.filePath
          ? currentTab.filePath.split(/[\\/]/).pop()
          : `${currentTab.title}.sql`;

        const result = await fileService.autoSaveQuery(
          fileName,
          tabData.queryText
        );

        if (result) {
          currentTab.filePath = result.path;
          currentTab.modified = false;
          context.updateTabs();

          // Add to recent files
          const fileName = result.path.split(/[\\/]/).pop();
          recentFilesStore.addFile(result.path, fileName);

          // Show success in status bar
          saveStatus.set({
            message: "Query saved",
            type: "success",
            timestamp: Date.now(),
          });

          // Clear status after 3 seconds
          setTimeout(() => {
            saveStatus.set({ message: null, type: null, timestamp: null });
          }, 3000);
        }
      } catch (error) {
        console.error("Failed to auto-save query:", error);

        // Import saveStatus store for error
        const { saveStatus } = await import("../stores/connections");
        saveStatus.set({
          message: `Save failed: ${error.message}`,
          type: "error",
          timestamp: Date.now(),
        });

        // Clear error after 5 seconds
        setTimeout(() => {
          saveStatus.set({ message: null, type: null, timestamp: null });
        }, 5000);
      } finally {
        // Always clear saving status
        const { isSaving } = await import("../stores/connections");
        isSaving.set(false);
      }
    },

    async handleSaveAs() {
      const currentTab = get(activeTab);
      if (!currentTab || currentTab.type !== "query") {
        await showError("No query tab is active");
        return;
      }

      const tabData = tabDataStore.get(currentTab.id);
      if (!tabData || !tabData.queryText) {
        await showError("No query to save");
        return;
      }

      try {
        // Import saveStatus store
        const { isSaving, saveStatus } = await import("../stores/connections");

        // Set saving status
        isSaving.set(true);
        saveStatus.set({
          message: "Saving...",
          type: "info",
          timestamp: Date.now(),
        });

        const result = await fileService.saveQueryAs(
          currentTab.title,
          tabData.queryText
        );

        if (result) {
          currentTab.filePath = result.path;
          currentTab.modified = false;
          currentTab.title = result.name;
          context.updateTabs();

          // Add to recent files
          const fileName = result.path.split(/[\\/]/).pop();
          recentFilesStore.addFile(result.path, fileName);

          // Show success in status bar
          saveStatus.set({
            message: "Query saved",
            type: "success",
            timestamp: Date.now(),
          });

          // Clear status after 3 seconds
          setTimeout(() => {
            saveStatus.set({ message: null, type: null, timestamp: null });
          }, 3000);
        }
      } catch (error) {
        console.error("Failed to save query:", error);

        // Import saveStatus store for error
        const { saveStatus } = await import("../stores/connections");
        saveStatus.set({
          message: `Save failed: ${error.message}`,
          type: "error",
          timestamp: Date.now(),
        });

        // Clear error after 5 seconds
        setTimeout(() => {
          saveStatus.set({ message: null, type: null, timestamp: null });
        }, 5000);
      } finally {
        // Always clear saving status
        const { isSaving } = await import("../stores/connections");
        isSaving.set(false);
      }
    },

    async handleExportData() {
      const currentTab = get(activeTab);
      if (!currentTab) {
        await showError("No active tab");
        return;
      }

      const tabData = tabDataStore.get(currentTab.id);
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
      const currentTab = get(activeTab);
      if (currentTab && currentTab.type === "query") {
        document.dispatchEvent(
          new CustomEvent("editor-undo", { detail: { tabId: currentTab.id } })
        );
      }
    },

    handleRedo() {
      const currentTab = get(activeTab);
      if (currentTab && currentTab.type === "query") {
        document.dispatchEvent(
          new CustomEvent("editor-redo", { detail: { tabId: currentTab.id } })
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
      const currentTab = get(activeTab);
      if (currentTab && currentTab.type === "query") {
        try {
          const text = await navigator.clipboard.readText();
          document.dispatchEvent(
            new CustomEvent("editor-paste", {
              detail: { tabId: currentTab.id, text },
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
      const currentTab = get(activeTab);
      if (!currentTab) {
        await showError("No active tab");
        return;
      }

      const tabData = tabDataStore.get(currentTab.id);
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
      const currentTab = get(activeTab);
      if (!currentTab || currentTab.type !== "query") {
        await showError("No query tab is active");
        return;
      }

      document.dispatchEvent(
        new CustomEvent("execute-query", {
          detail: { tabId: currentTab.id },
        })
      );
    },

    async handleExecuteScript() {
      const currentTab = get(activeTab);
      if (!currentTab || currentTab.type !== "query") {
        await showError("No query tab is active");
        return;
      }

      document.dispatchEvent(
        new CustomEvent("execute-script", {
          detail: { tabId: currentTab.id },
        })
      );
    },

    handleStop() {
      const currentTab = get(activeTab);
      if (!currentTab) return;

      const controller = runningQueries.get(currentTab.id);
      if (controller) {
        controller.abort();
        runningQueries.delete(currentTab.id);

        document.dispatchEvent(
          new CustomEvent("stop-query", {
            detail: { tabId: currentTab.id },
          })
        );
      }
    },

    async handleRefresh() {
      const currentTab = get(activeTab);
      if (!currentTab) {
        await showError("No active tab");
        return;
      }

      if (currentTab.type === "table") {
        try {
          const tableInfo = currentTab.tableInfo;

          const tableData = await loadTableData({
            connection_id: tableInfo.connection.id,
            query: {
              db_type: tableInfo.connection.db_type,
              database: tableInfo.database,
              schema: tableInfo.schema || null,
              table: tableInfo.name,
              limit: 200,
              offset: 0,
              filters: null,
              order_by: null,
            },
          });

          tabDataStore.setQueryResult(currentTab.id, tableData);
          await showMessage("Table data refreshed");
        } catch (error) {
          console.error("Failed to refresh table data:", error);
          await showError("Failed to refresh: " + error.message);
        }
      } else if (currentTab.type === "query") {
        const tabData = tabDataStore.get(currentTab.id);
        if (tabData && tabData.executedQuery) {
          document.dispatchEvent(
            new CustomEvent("execute-query", {
              detail: { tabId: currentTab.id },
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
