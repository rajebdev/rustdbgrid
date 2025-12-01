import { writable, get } from "svelte/store";
import { tabDataStore } from "./tabData";

/**
 * Tab Management Store and Logic
 */
function createTabStore() {
  const STORAGE_KEY = "rustdbgrid_tabs";
  const ACTIVE_TAB_KEY = "rustdbgrid_active_tab";

  // Load initial tabs from localStorage
  let initialTabs = [];
  let initialActiveTab = null;

  if (typeof window !== "undefined") {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        initialTabs = JSON.parse(saved);
      }
      const savedActive = localStorage.getItem(ACTIVE_TAB_KEY);
      if (savedActive) {
        initialActiveTab = JSON.parse(savedActive);
      }
    } catch (e) {
      console.error("Failed to load tabs from storage:", e);
      initialTabs = [];
      initialActiveTab = null;
    }
  }

  const tabs = writable(initialTabs);
  const activeTab = writable(initialActiveTab);

  // Auto-save tabs to localStorage whenever they change
  tabs.subscribe((currentTabs) => {
    if (typeof window !== "undefined") {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(currentTabs));
    }
  });

  // Auto-save active tab to localStorage
  activeTab.subscribe((currentActive) => {
    if (typeof window !== "undefined") {
      if (currentActive) {
        localStorage.setItem(ACTIVE_TAB_KEY, JSON.stringify(currentActive));
      } else {
        localStorage.removeItem(ACTIVE_TAB_KEY);
      }
    }
  });

  return {
    subscribe: tabs.subscribe,
    activeTab,

    /**
     * Validate tabs (check if tabs are still valid)
     * Can be called on app startup to ensure tabs are properly loaded
     */
    validateTabs: () => {
      const currentTabs = get(tabs);
      const currentActive = get(activeTab);

      // Check if active tab still exists in tabs
      if (
        currentActive &&
        !currentTabs.find((t) => t.id === currentActive.id)
      ) {
        if (currentTabs.length > 0) {
          activeTab.set(currentTabs[0]);
        } else {
          activeTab.set(null);
        }
      }
    },

    /**
     * Add a new query tab
     */
    addQueryTab: (connection = null, initialContent = null) => {
      tabs.update((currentTabs) => {
        const newTab = {
          id: Date.now(),
          title: `Query ${currentTabs.length + 1}`,
          type: "query",
          modified: false,
          connection: connection, // Store connection info
          initialContent: initialContent, // Store initial content to pass to components
        };

        // Initialize tabDataStore with initial content if provided
        if (initialContent) {
          tabDataStore.setQueryText(newTab.id, initialContent);
        }

        activeTab.set(newTab);
        return [...currentTabs, newTab];
      });
    },

    /**
     * Add a new table tab
     */
    addTableTab: (table, database, connection) => {
      tabs.update((currentTabs) => {
        // Check if tab already exists (including connection ID to support same table names across different connections)
        const existingTab = currentTabs.find(
          (t) =>
            t.type === "table" &&
            t.tableInfo?.name === table.name &&
            t.tableInfo?.schema === table.schema &&
            t.tableInfo?.database === database.name &&
            t.tableInfo?.connection?.id === connection.id
        );

        if (existingTab) {
          activeTab.set(existingTab);
          return currentTabs;
        }

        // Create new tab
        const displayName =
          connection.db_type === "PostgreSQL" && table.schema
            ? `${table.schema}.${table.name}`
            : table.name;

        const newTab = {
          id: Date.now(),
          title: displayName,
          type: "table",
          modified: false,
          tableInfo: {
            name: table.name,
            schema: table.schema,
            database: database.name,
            connection: connection,
          },
        };

        activeTab.set(newTab);
        return [...currentTabs, newTab];
      });
    },

    /**
     * Add a new procedure tab
     */
    addProcedureTab: (procedure, database, connection) => {
      tabs.update((currentTabs) => {
        // Check if tab already exists
        const existingTab = currentTabs.find(
          (t) =>
            t.type === "procedure" &&
            t.procedureInfo?.name === procedure.name &&
            t.procedureInfo?.schema === procedure.schema &&
            t.procedureInfo?.database === database.name
        );

        if (existingTab) {
          activeTab.set(existingTab);
          return currentTabs;
        }

        // Create new tab - include schema in display name for PostgreSQL/MSSQL
        const displayName =
          (connection.db_type === "PostgreSQL" ||
            connection.db_type === "MSSQL") &&
          procedure.schema
            ? `${procedure.schema}.${procedure.name}`
            : procedure.name;

        const newTab = {
          id: Date.now(),
          title: displayName,
          type: "procedure",
          modified: false,
          procedureInfo: {
            name: procedure.name,
            schema: procedure.schema,
            procedure_type: procedure.procedure_type,
            database: database.name,
            connection: connection,
          },
        };

        activeTab.set(newTab);
        return [...currentTabs, newTab];
      });
    },

    /**
     * Close a tab
     */
    closeTab: (tabToClose) => {
      let tabIndex = -1;

      tabs.update((currentTabs) => {
        tabIndex = currentTabs.findIndex((t) => t.id === tabToClose.id);
        const newTabs = currentTabs.filter((t) => t.id !== tabToClose.id);

        // Update active tab if needed
        const currentActive = get(activeTab);
        if (currentActive?.id === tabToClose.id) {
          if (newTabs.length > 0) {
            if (tabIndex < newTabs.length) {
              activeTab.set(newTabs[tabIndex]);
            } else {
              activeTab.set(newTabs[tabIndex - 1]);
            }
          } else {
            activeTab.set(null);
          }
        }

        return newTabs;
      });
    },

    /**
     * Select a tab
     */
    selectTab: (tab) => {
      activeTab.set(tab);
    },

    /**
     * Go to next tab
     */
    nextTab: () => {
      const currentTabs = get(tabs);
      const currentActive = get(activeTab);

      if (currentTabs.length > 1 && currentActive) {
        const currentIndex = currentTabs.findIndex(
          (t) => t.id === currentActive.id
        );
        const nextIndex = (currentIndex + 1) % currentTabs.length;
        activeTab.set(currentTabs[nextIndex]);
      }
    },

    /**
     * Go to previous tab
     */
    previousTab: () => {
      const currentTabs = get(tabs);
      const currentActive = get(activeTab);

      if (currentTabs.length > 1 && currentActive) {
        const currentIndex = currentTabs.findIndex(
          (t) => t.id === currentActive.id
        );
        const prevIndex =
          (currentIndex - 1 + currentTabs.length) % currentTabs.length;
        activeTab.set(currentTabs[prevIndex]);
      }
    },

    /**
     * Update tabs array (for reactivity)
     */
    updateTabs: () => {
      tabs.update((currentTabs) => [...currentTabs]);
    },

    /**
     * Update a specific tab
     */
    updateTab: (updatedTab) => {
      tabs.update((currentTabs) => {
        return currentTabs.map((t) =>
          t.id === updatedTab.id ? updatedTab : t
        );
      });
    },

    /**
     * Set tabs
     */
    setTabs: (newTabs) => {
      tabs.set(newTabs);
    },

    /**
     * Close all tabs for a specific connection
     * Returns array of closed tab IDs for cleanup
     */
    closeTabsByConnection: (connectionId) => {
      let closedTabIds = [];

      tabs.update((currentTabs) => {
        const tabsToKeep = currentTabs.filter((tab) => {
          // For query tabs, check tab.connection
          if (tab.type === "query" && tab.connection?.id === connectionId) {
            closedTabIds.push(tab.id);
            return false;
          }
          // For table tabs, check tab.tableInfo.connection
          if (
            tab.type === "table" &&
            tab.tableInfo?.connection?.id === connectionId
          ) {
            closedTabIds.push(tab.id);
            return false;
          }
          // For procedure tabs, check tab.procedureInfo.connection
          if (
            tab.type === "procedure" &&
            tab.procedureInfo?.connection?.id === connectionId
          ) {
            closedTabIds.push(tab.id);
            return false;
          }
          return true;
        });

        // Update active tab if current active tab was closed
        const currentActive = get(activeTab);
        if (currentActive) {
          const isActiveTabClosed = !tabsToKeep.find(
            (t) => t.id === currentActive.id
          );
          if (isActiveTabClosed) {
            if (tabsToKeep.length > 0) {
              activeTab.set(tabsToKeep[0]);
            } else {
              activeTab.set(null);
            }
          }
        }

        return tabsToKeep;
      });

      return closedTabIds;
    },

    /**
     * Clear all tabs and active tab (useful for cleanup)
     */
    clearAll: () => {
      tabs.set([]);
      activeTab.set(null);
      if (typeof window !== "undefined") {
        localStorage.removeItem(STORAGE_KEY);
        localStorage.removeItem(ACTIVE_TAB_KEY);
      }
    },
  };
}

export const tabStore = createTabStore();
