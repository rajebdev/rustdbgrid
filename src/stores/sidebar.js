import { writable, derived } from "svelte/store";

// Centralized sidebar state management
function createSidebarStore() {
  const { subscribe, set, update } = writable({
    // Search
    searchQuery: "",

    // Expanded states
    expandedConnections: {},
    expandedDatabases: {},
    expandedSchemas: {},
    expandedSchemasParent: {},
    expandedGroups: {},

    // Loading states
    loadingConnections: {},
    loadingDatabases: {},
    loadingSchemas: {},

    // Connection status
    connectedConnections: {},

    // Cached data
    cachedData: {},

    // Context menu state (unified)
    contextMenu: null,

    // Active items
    activeContextItem: null,

    // Modal states
    showConnectionModal: false,
    editingConnection: null,
    showRenameModal: false,
    renameModalData: null,
  });

  return {
    subscribe,

    // Search
    setSearchQuery: (query) =>
      update((state) => ({ ...state, searchQuery: query })),

    // Expanded states
    toggleConnection: (connId, data = null) =>
      update((state) => {
        const newExpanded = { ...state.expandedConnections };
        if (newExpanded[connId]) {
          delete newExpanded[connId];
        } else {
          newExpanded[connId] = data;
        }
        return { ...state, expandedConnections: newExpanded };
      }),

    toggleDatabase: (key, expand = null) =>
      update((state) => {
        const newExpanded = { ...state.expandedDatabases };
        if (expand === null) {
          newExpanded[key] = !newExpanded[key];
        } else {
          if (expand) {
            newExpanded[key] = true;
          } else {
            delete newExpanded[key];
          }
        }
        return { ...state, expandedDatabases: newExpanded };
      }),

    toggleSchema: (key, expand = null) =>
      update((state) => {
        const newExpanded = { ...state.expandedSchemas };
        if (expand === null) {
          newExpanded[key] = !newExpanded[key];
        } else {
          if (expand) {
            newExpanded[key] = true;
          } else {
            delete newExpanded[key];
          }
        }
        return { ...state, expandedSchemas: newExpanded };
      }),

    toggleSchemasParent: (key) =>
      update((state) => ({
        ...state,
        expandedSchemasParent: {
          ...state.expandedSchemasParent,
          [key]: !state.expandedSchemasParent[key],
        },
      })),

    toggleGroup: (key) =>
      update((state) => ({
        ...state,
        expandedGroups: {
          ...state.expandedGroups,
          [key]: !state.expandedGroups[key],
        },
      })),

    // Loading states
    setConnectionLoading: (connId, loading) =>
      update((state) => ({
        ...state,
        loadingConnections: { ...state.loadingConnections, [connId]: loading },
      })),

    setDatabaseLoading: (key, loading) =>
      update((state) => ({
        ...state,
        loadingDatabases: { ...state.loadingDatabases, [key]: loading },
      })),

    setSchemaLoading: (key, loading) =>
      update((state) => ({
        ...state,
        loadingSchemas: { ...state.loadingSchemas, [key]: loading },
      })),

    // Connection status
    setConnectedConnections: (connections) =>
      update((state) => ({
        ...state,
        connectedConnections: connections,
      })),

    setConnectionConnected: (connId, connected) =>
      update((state) => ({
        ...state,
        connectedConnections: {
          ...state.connectedConnections,
          [connId]: connected,
        },
      })),

    // Cache management
    setCachedData: (key, data) =>
      update((state) => ({
        ...state,
        cachedData: { ...state.cachedData, [key]: data },
      })),

    updateCachedData: (key, updater) =>
      update((state) => {
        const currentData = state.cachedData[key] || {};
        return {
          ...state,
          cachedData: {
            ...state.cachedData,
            [key]:
              typeof updater === "function"
                ? updater(currentData)
                : { ...currentData, ...updater },
          },
        };
      }),

    clearCachedData: (key) =>
      update((state) => {
        const newCache = { ...state.cachedData };
        delete newCache[key];
        return { ...state, cachedData: newCache };
      }),

    clearAllCache: () => update((state) => ({ ...state, cachedData: {} })),

    // Context menu (unified)
    openContextMenu: (menuType, data) =>
      update((state) => ({
        ...state,
        contextMenu: { type: menuType, ...data },
        activeContextItem: data.itemId || null,
      })),

    closeContextMenu: () =>
      update((state) => ({
        ...state,
        contextMenu: null,
        activeContextItem: null,
      })),

    // Modals
    openConnectionModal: (connection = null) =>
      update((state) => ({
        ...state,
        showConnectionModal: true,
        editingConnection: connection,
      })),

    closeConnectionModal: () =>
      update((state) => ({
        ...state,
        showConnectionModal: false,
        editingConnection: null,
      })),

    openRenameModal: (title, value, callback) =>
      update((state) => ({
        ...state,
        showRenameModal: true,
        renameModalData: { title, value, callback },
      })),

    closeRenameModal: () =>
      update((state) => ({
        ...state,
        showRenameModal: false,
        renameModalData: null,
      })),

    // Reset all state
    reset: () =>
      set({
        searchQuery: "",
        expandedConnections: {},
        expandedDatabases: {},
        expandedSchemas: {},
        expandedSchemasParent: {},
        expandedGroups: {},
        loadingConnections: {},
        loadingDatabases: {},
        loadingSchemas: {},
        connectedConnections: {},
        cachedData: {},
        contextMenu: null,
        activeContextItem: null,
        showConnectionModal: false,
        editingConnection: null,
        showRenameModal: false,
        renameModalData: null,
      }),
  };
}

export const sidebarStore = createSidebarStore();

// Derived stores for convenience
export const expandedConnections = derived(
  sidebarStore,
  ($s) => $s.expandedConnections
);
export const expandedDatabases = derived(
  sidebarStore,
  ($s) => $s.expandedDatabases
);
export const expandedSchemas = derived(
  sidebarStore,
  ($s) => $s.expandedSchemas
);
export const cachedData = derived(sidebarStore, ($s) => $s.cachedData);
export const contextMenu = derived(sidebarStore, ($s) => $s.contextMenu);
