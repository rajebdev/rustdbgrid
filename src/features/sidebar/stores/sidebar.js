import { writable, derived } from "svelte/store";

/**
 * Creates a generic toggle handler for expanding/collapsing items
 * @param {Function} update - The store's update function
 * @param {string} stateKey - The key in state to toggle (e.g., 'expandedConnections')
 * @param {Object} options - Configuration options
 * @param {boolean} options.deleteOnClose - Whether to delete the key when closing (default: false)
 * @param {boolean} options.supportExplicitControl - Whether to support explicit expand parameter (default: false)
 * @returns {Function} - The toggle handler function
 */
function createToggleHandler(update, stateKey, options = {}) {
  const { deleteOnClose = false, supportExplicitControl = false } = options;

  return (key, expandOrData = null) => {
    return update((state) => {
      const newExpanded = { ...state[stateKey] };

      if (supportExplicitControl && typeof expandOrData === "boolean") {
        // Explicit control mode
        if (expandOrData) {
          newExpanded[key] = true;
        } else {
          delete newExpanded[key];
        }
      } else if (deleteOnClose) {
        // Delete-on-close mode (for connections)
        if (newExpanded[key] && expandOrData === null) {
          // Only delete if explicitly closing (null means toggle to close)
          delete newExpanded[key];
        } else if (expandOrData !== null) {
          // If data is provided, update/set it (even if already expanded)
          newExpanded[key] = expandOrData;
        }
      } else {
        // Simple toggle mode
        newExpanded[key] = !newExpanded[key];
      }

      return { ...state, [stateKey]: newExpanded };
    });
  };
}

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

    // Expanded states - using generic toggle handlers
    toggleConnection: createToggleHandler(update, "expandedConnections", {
      deleteOnClose: true,
    }),

    toggleDatabase: createToggleHandler(update, "expandedDatabases", {
      supportExplicitControl: true,
    }),

    toggleSchema: createToggleHandler(update, "expandedSchemas", {
      supportExplicitControl: true,
    }),

    toggleSchemasParent: createToggleHandler(update, "expandedSchemasParent"),

    toggleGroup: createToggleHandler(update, "expandedGroups"),

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
export const cachedData = derived(sidebarStore, ($s) => $s.cachedData);
export const contextMenu = derived(sidebarStore, ($s) => $s.contextMenu);
