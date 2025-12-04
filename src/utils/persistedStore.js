/**
 * Persisted Store Utility
 * Creates Svelte writable stores that automatically persist to localStorage
 */
import { writable } from "svelte/store";

/**
 * Create a Svelte store that persists to localStorage
 * @param {string} key - The localStorage key
 * @param {*} initialValue - The initial value if nothing is in localStorage
 * @param {Object} options - Optional configuration
 * @param {Function} options.transform - Transform function to apply before saving
 * @param {Function} options.deserialize - Custom deserialize function (default: JSON.parse)
 * @param {Function} options.serialize - Custom serialize function (default: JSON.stringify)
 * @returns {import('svelte/store').Writable} A Svelte writable store
 */
export function createPersistedStore(key, initialValue, options = {}) {
  const {
    transform = null,
    deserialize = JSON.parse,
    serialize = JSON.stringify,
  } = options;

  // Initialize the store
  const store = writable(initialValue);

  // Load from localStorage on initialization (browser only)
  if (typeof window !== "undefined") {
    const stored = localStorage.getItem(key);
    if (stored !== null) {
      try {
        const parsedValue = deserialize(stored);
        store.set(parsedValue);
      } catch (e) {
        console.error(`Failed to load ${key} from localStorage:`, e);
      }
    }
  }

  // Auto-save to localStorage on every change
  store.subscribe((value) => {
    if (typeof window !== "undefined") {
      try {
        // Apply transform if provided
        const valueToSave = transform ? transform(value) : value;
        localStorage.setItem(key, serialize(valueToSave));
      } catch (e) {
        console.error(`Failed to save ${key} to localStorage:`, e);
      }
    }
  });

  return store;
}
