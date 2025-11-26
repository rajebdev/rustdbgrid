import { writable } from "svelte/store";

export const connections = writable([]);
export const activeConnection = writable(null);
export const queryResults = writable(null);
export const selectedDatabase = writable(null);
export const selectedTable = writable(null);
export const isSaving = writable(false);
