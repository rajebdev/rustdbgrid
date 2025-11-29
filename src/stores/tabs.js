import { writable, get } from "svelte/store";

/**
 * Tab Management Store and Logic
 */
function createTabStore() {
  const tabs = writable([]);
  const activeTab = writable(null);

  return {
    subscribe: tabs.subscribe,
    activeTab,

    /**
     * Add a new query tab
     */
    addQueryTab: (connection = null) => {
      tabs.update((currentTabs) => {
        const newTab = {
          id: Date.now(),
          title: `Query ${currentTabs.length + 1}`,
          type: "query",
          modified: false,
          connection: connection, // Store connection info
        };
        activeTab.set(newTab);
        return [...currentTabs, newTab];
      });
    },

    /**
     * Add a new table tab
     */
    addTableTab: (table, database, connection) => {
      tabs.update((currentTabs) => {
        // Check if tab already exists
        const existingTab = currentTabs.find(
          (t) =>
            t.type === "table" &&
            t.tableInfo?.name === table.name &&
            t.tableInfo?.schema === table.schema &&
            t.tableInfo?.database === database.name
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
            t.procedureInfo?.database === database.name
        );

        if (existingTab) {
          activeTab.set(existingTab);
          return currentTabs;
        }

        // Create new tab
        const newTab = {
          id: Date.now(),
          title: `${database.name}.${procedure.name}`,
          type: "procedure",
          modified: false,
          procedureInfo: {
            name: procedure.name,
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
  };
}

export const tabStore = createTabStore();
