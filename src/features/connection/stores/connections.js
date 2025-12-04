import { writable } from "svelte/store";
import { createPersistedStore } from "../utils/persistedStore";

export const connections = writable([]);
export const activeConnection = createPersistedStore("activeConnection", null);
export const selectedDatabase = createPersistedStore("selectedDatabase", null);
export const selectedTable = writable(null);
export const isSaving = writable(false);
export const saveStatus = writable({
  message: null,
  type: null,
  timestamp: null,
});
