import { writable, get as getStoreValue } from "svelte/store";

// Store untuk menyimpan data per tab
// Format: { [tabId]: { queryResult, queryText, executedQuery, filters, sortColumn, sortDirection, scrollPosition, viewMode } }
const createTabDataStore = () => {
  const STORAGE_KEY = "rustdbgrid_tab_data";

  // Load initial data from localStorage
  let initialData = {};
  if (typeof window !== "undefined") {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        initialData = JSON.parse(saved);
      }
    } catch (e) {
      console.error("Failed to load tab data from storage:", e);
      initialData = {};
    }
  }

  const store = writable(initialData);
  const { subscribe, set, update } = store;

  // Auto-save to localStorage whenever data changes
  subscribe((data) => {
    if (typeof window !== "undefined") {
      // Only save query-related data, not large result sets
      const dataToSave = {};
      Object.keys(data).forEach((tabId) => {
        dataToSave[tabId] = {
          queryText: data[tabId].queryText,
          executedQuery: data[tabId].executedQuery,
          viewMode: data[tabId].viewMode,
          activeSubTab: data[tabId].activeSubTab,
          activePropertiesTab: data[tabId].activePropertiesTab,
        };
      });
      localStorage.setItem(STORAGE_KEY, JSON.stringify(dataToSave));
    }
  });

  return {
    subscribe,

    // Set query result untuk tab tertentu
    setQueryResult: (tabId, queryResult) => {
      update((store) => ({
        ...store,
        [tabId]: {
          ...store[tabId],
          queryResult,
        },
      }));
    },

    // Set query text untuk tab tertentu
    setQueryText: (tabId, queryText) => {
      update((store) => ({
        ...store,
        [tabId]: {
          ...store[tabId],
          queryText,
        },
      }));
    },

    // Set executed query untuk tab tertentu
    setExecutedQuery: (tabId, executedQuery) => {
      update((store) => ({
        ...store,
        [tabId]: {
          ...store[tabId],
          executedQuery,
        },
      }));
    },

    // Set filter state untuk tab tertentu
    setFilters: (tabId, filters) => {
      update((store) => ({
        ...store,
        [tabId]: {
          ...store[tabId],
          filters,
        },
      }));
    },

    // Set sort state untuk tab tertentu
    setSortStack: (tabId, sortStack) => {
      update((store) => ({
        ...store,
        [tabId]: {
          ...store[tabId],
          sortStack,
        },
      }));
    },

    // Set scroll position untuk tab tertentu
    setScrollPosition: (tabId, scrollPosition) => {
      update((store) => ({
        ...store,
        [tabId]: {
          ...store[tabId],
          scrollPosition,
        },
      }));
    },

    // Set view mode untuk tab tertentu (grid atau json)
    setViewMode: (tabId, viewMode) => {
      update((store) => ({
        ...store,
        [tabId]: {
          ...store[tabId],
          viewMode,
        },
      }));
    },

    // Set error state untuk tab tertentu
    setError: (tabId, error) => {
      update((store) => ({
        ...store,
        [tabId]: {
          ...store[tabId],
          error,
        },
      }));
    },

    // Clear error state untuk tab tertentu
    clearError: (tabId) => {
      update((store) => ({
        ...store,
        [tabId]: {
          ...store[tabId],
          error: null,
        },
      }));
    },

    // Set active subtab untuk table tab tertentu
    setActiveSubTab: (tabId, activeSubTab) => {
      update((store) => ({
        ...store,
        [tabId]: {
          ...store[tabId],
          activeSubTab,
        },
      }));
    },

    // Set active properties tab untuk table tab tertentu
    setActivePropertiesTab: (tabId, activePropertiesTab) => {
      update((store) => ({
        ...store,
        [tabId]: {
          ...store[tabId],
          activePropertiesTab,
        },
      }));
    },

    // Get data untuk tab tertentu
    get: (tabId) => {
      const storeData = getStoreValue(store);
      return (
        storeData[tabId] || {
          queryResult: null,
          queryText: "",
          executedQuery: "",
          filters: {},
          sortColumn: null,
          sortDirection: "asc",
          scrollPosition: 0,
          viewMode: "grid",
          activeSubTab: "data",
          activePropertiesTab: "Columns",
        }
      );
    },

    // Get data untuk tab tertentu (alias untuk kompatibilitas)
    getTabData: (tabId) => {
      const storeData = getStoreValue(store);
      return (
        storeData[tabId] || {
          queryResult: null,
          queryText: "",
          executedQuery: "",
          filters: {},
          sortColumn: null,
          sortDirection: "asc",
          scrollPosition: 0,
          viewMode: "grid",
          activeSubTab: "data",
          activePropertiesTab: "Columns",
        }
      );
    },

    // Hapus data tab ketika tab ditutup
    removeTab: (tabId) => {
      update((store) => {
        const newStore = { ...store };
        delete newStore[tabId];
        return newStore;
      });
    },

    // Hapus data untuk multiple tabs (by tab IDs)
    removeTabsByIds: (tabIds) => {
      update((store) => {
        const newStore = { ...store };
        tabIds.forEach((tabId) => {
          delete newStore[tabId];
        });
        return newStore;
      });
    },

    // Clear all data and localStorage
    clear: () => {
      set({});
      if (typeof window !== "undefined") {
        localStorage.removeItem(STORAGE_KEY);
      }
    },
  };
};

export const tabDataStore = createTabDataStore();
