import { writable } from "svelte/store";

/**
 * Query History Store - untuk menyimpan history queries yang dijalankan
 */
function createQueryHistoryStore() {
  const STORAGE_KEY = "rustdbgrid_query_history";
  const MAX_HISTORY = 100;

  const queryHistory = writable([]);

  // Load from localStorage on init
  if (typeof window !== "undefined") {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      try {
        queryHistory.set(JSON.parse(saved));
      } catch (e) {
        console.error("Failed to load query history:", e);
      }
    }
  }

  function saveToStorage(history) {
    if (typeof window !== "undefined") {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(history));
    }
  }

  return {
    subscribe: queryHistory.subscribe,

    addToHistory: (query, connectionId, databaseName, executionTime = 0) => {
      queryHistory.update((history) => {
        const newEntry = {
          id: Date.now(),
          query,
          connectionId,
          databaseName,
          executedAt: new Date().toISOString(),
          executionTime,
        };

        const updated = [newEntry, ...history].slice(0, MAX_HISTORY);
        saveToStorage(updated);
        return updated;
      });
    },

    clearHistory: () => {
      queryHistory.set([]);
      if (typeof window !== "undefined") {
        localStorage.removeItem(STORAGE_KEY);
      }
    },

    deleteEntry: (id) => {
      queryHistory.update((history) => {
        const updated = history.filter((h) => h.id !== id);
        saveToStorage(updated);
        return updated;
      });
    },
  };
}

export const queryHistoryStore = createQueryHistoryStore();
