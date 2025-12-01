import { writable } from "svelte/store";

// Initialize from localStorage
function createPersistentStore(key, initialValue) {
  const store = writable(initialValue);

  if (typeof window !== "undefined") {
    const stored = localStorage.getItem(key);
    if (stored) {
      try {
        store.set(JSON.parse(stored));
      } catch (e) {
        console.error(`Failed to load ${key} from localStorage:`, e);
      }
    }
  }

  // Auto-save to localStorage
  store.subscribe((value) => {
    if (typeof window !== "undefined") {
      localStorage.setItem(key, JSON.stringify(value));
    }
  });

  return store;
}

export const connections = writable([]);
export const activeConnection = createPersistentStore("activeConnection", null);
export const queryResults = writable(null);
export const selectedDatabase = createPersistentStore("selectedDatabase", null);
export const selectedTable = writable(null);
export const isSaving = writable(false);
export const saveStatus = writable({
  message: null,
  type: null,
  timestamp: null,
});
