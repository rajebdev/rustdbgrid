import {
  fileService,
  showMessage,
  showError,
} from "../../features/settings/services/fileService";
import { get } from "svelte/store";
import {
  getNextQueryNumber,
  disconnectFromDatabase,
} from "../../core/integrations/tauri";
import { recentFilesStore } from "../../features/settings/stores/recentFiles";
import { invoke } from "@tauri-apps/api/core";
import { readTextFile } from "@tauri-apps/plugin-fs";
import { open as openUrl } from "@tauri-apps/plugin-shell";
import {
  isSaving,
  saveStatus,
} from "../../features/connection/stores/connections";

/**
 * Handle opening table tab
 * DataGrid will auto-load table data when it mounts
 */
export async function handleOpenTableTab(event, tabStore, tabDataStore) {
  const { table, database, connection } = event.detail;

  tabStore.addTableTab(table, database, connection);
}

/**
 * Handle opening procedure tab (shows procedure source code)
 */
export async function handleOpenProcedureTab(event, tabStore) {
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
    activeConnection,
    showToolbar,
    runningQueries,
  } = context;

  // Get activeTab from tabStore (it's a derived store)
  const activeTab = tabStore.activeTab;

  return {
    // File Menu Handlers
    async handleNewQuery() {
      try {
        console.log("Creating new query tab...");
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

        // Get the newly created tab and update its title
        const newTab = get(activeTab);
        if (newTab) {
          // Create updated tab object with new properties
          const updatedTab = {
            ...newTab,
            title: fileName,
            filePath: null, // No file path until first save
            modified: false, // Not modified yet - waiting for user changes
            initialContent: newContent, // Store initial template content
          };

          // Save the updated tab back to the store
          tabStore.updateTab(updatedTab);
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
            const updatedTab = {
              ...newTab,
              filePath: file.path,
              title: file.name.replace(/\.[^/.]+$/, ""), // Remove extension
              modified: false, // File just opened, not modified
              initialContent: file.content, // Store initial content
            };
            tabStore.updateTab(updatedTab);
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
        const content = await readTextFile(file.path);

        // Update recent files (move to top)
        recentFilesStore.addFile(file.path, file.name);

        // Create new query tab
        tabStore.addQueryTab(get(activeConnection), content);

        // Get the newly created tab
        const newTab = get(tabStore.activeTab);
        if (newTab) {
          // Update with file info
          const updatedTab = {
            ...newTab,
            filePath: file.path,
            title: file.name.replace(/\.[^/.]+$/, ""), // Remove extension
            modified: false, // File just opened, not modified
            initialContent: content, // Store initial content
          };
          tabStore.updateTab(updatedTab);
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
          // Update tab with saved state
          const updatedTab = {
            ...currentTab,
            filePath: result.path,
            modified: false,
            initialContent: tabData.queryText, // Update initial content to saved content
          };
          tabStore.updateTab(updatedTab);

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

        // Set error status
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
        isSaving.set(false);
      }
    },

    async handleSaveQueryAs() {
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
          // Update tab with saved state
          const updatedTab = {
            ...currentTab,
            filePath: result.path,
            modified: false,
            title: result.name,
            initialContent: tabData.queryText, // Update initial content to saved content
          };
          tabStore.updateTab(updatedTab);

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

        // Set error status
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
        // Trigger DataGrid to reload data
        document.dispatchEvent(
          new CustomEvent("reload-datagrid", {
            detail: { tabId: currentTab.id },
          })
        );
        await showMessage("Refreshing table data...");
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
        await openUrl("https://github.com/yourusername/rustdbgrid#readme");
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
