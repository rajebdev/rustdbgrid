import { writable } from "svelte/store";

/**
 * Query List Store - untuk menyimpan queries favorit
 */
function createQueryListStore() {
  const STORAGE_KEY = "rustdbgrid_saved_queries";

  const queryList = writable([]);

  // Load from localStorage on init
  if (typeof window !== "undefined") {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      try {
        queryList.set(JSON.parse(saved));
      } catch (e) {
        console.error("Failed to load saved queries:", e);
      }
    }
  }

  function saveToStorage(queries) {
    if (typeof window !== "undefined") {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(queries));
    }
  }

  return {
    subscribe: queryList.subscribe,

    /**
     * Add a new query to the list
     */
    addQuery: (title, content, description = "") => {
      queryList.update((queries) => {
        const newQuery = {
          id: Date.now(),
          title: title || `Query ${queries.length + 1}`,
          content,
          description,
          createdAt: new Date().toISOString(),
          lastUsed: new Date().toISOString(),
        };
        const updated = [newQuery, ...queries];
        saveToStorage(updated);
        return updated;
      });
    },

    /**
     * Update an existing query
     */
    updateQuery: (id, updates) => {
      queryList.update((queries) => {
        const updated = queries.map((q) =>
          q.id === id ? { ...q, ...updates } : q
        );
        saveToStorage(updated);
        return updated;
      });
    },

    /**
     * Delete a query
     */
    deleteQuery: (id) => {
      queryList.update((queries) => {
        const updated = queries.filter((q) => q.id !== id);
        saveToStorage(updated);
        return updated;
      });
    },

    /**
     * Get a query by ID
     */
    getQuery: (id) => {
      let result = null;
      queryList.subscribe((queries) => {
        result = queries.find((q) => q.id === id);
      })();
      return result;
    },

    /**
     * Update lastUsed timestamp
     */
    markUsed: (id) => {
      queryList.update((queries) => {
        const updated = queries.map((q) =>
          q.id === id ? { ...q, lastUsed: new Date().toISOString() } : q
        );
        saveToStorage(updated);
        return updated;
      });
    },

    /**
     * Clear all queries
     */
    clearAll: () => {
      queryList.set([]);
      if (typeof window !== "undefined") {
        localStorage.removeItem(STORAGE_KEY);
      }
    },
  };
}

export const queryListStore = createQueryListStore();
