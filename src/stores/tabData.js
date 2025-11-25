import { writable } from "svelte/store";

// Store untuk menyimpan data per tab
// Format: { [tabId]: { queryResult, queryText, executedQuery, filters, sortColumn, sortDirection, scrollPosition } }
const createTabDataStore = () => {
  const { subscribe, set, update } = writable({});

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
    setSortState: (tabId, sortColumn, sortDirection) => {
      update((store) => ({
        ...store,
        [tabId]: {
          ...store[tabId],
          sortColumn,
          sortDirection,
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

    // Get data untuk tab tertentu
    getTabData: (tabId) => {
      let data = null;
      subscribe((store) => {
        data = store[tabId] || {
          queryResult: null,
          queryText: "",
          executedQuery: "",
          filters: {},
          sortColumn: null,
          sortDirection: "asc",
          scrollPosition: 0,
        };
      })();
      return data;
    },

    // Hapus data tab ketika tab ditutup
    removeTab: (tabId) => {
      update((store) => {
        const newStore = { ...store };
        delete newStore[tabId];
        return newStore;
      });
    },

    // Clear all data
    clear: () => set({}),
  };
};

export const tabDataStore = createTabDataStore();
