import { writable } from "svelte/store";

/**
 * Default pagination limit for data grid and query results
 * User can change this in settings
 */
export const defaultPaginateLimit = writable(200);
