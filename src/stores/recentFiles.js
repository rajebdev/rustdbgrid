import { writable } from "svelte/store";

/**
 * Recent Files Store - stores recently opened query files
 */
function createRecentFilesStore() {
  const STORAGE_KEY = "rustdbgrid_recent_files";
  const MAX_RECENT = 10;

  const recentFiles = writable([]);

  // Load from localStorage on init
  if (typeof window !== "undefined") {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      try {
        recentFiles.set(JSON.parse(saved));
      } catch (e) {
        console.error("Failed to load recent files:", e);
      }
    }
  }

  function saveToStorage(files) {
    if (typeof window !== "undefined") {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(files));
    }
  }

  return {
    subscribe: recentFiles.subscribe,

    /**
     * Add a file to recent files list
     */
    addFile: (filePath, fileName) => {
      recentFiles.update((files) => {
        // Remove if already exists
        const filtered = files.filter((f) => f.path !== filePath);

        const newFile = {
          path: filePath,
          name: fileName,
          openedAt: new Date().toISOString(),
        };

        // Add to front and limit to MAX_RECENT
        const updated = [newFile, ...filtered].slice(0, MAX_RECENT);
        saveToStorage(updated);
        return updated;
      });
    },

    /**
     * Remove a file from recent files
     */
    removeFile: (filePath) => {
      recentFiles.update((files) => {
        const updated = files.filter((f) => f.path !== filePath);
        saveToStorage(updated);
        return updated;
      });
    },

    /**
     * Clear all recent files
     */
    clearAll: () => {
      recentFiles.set([]);
      if (typeof window !== "undefined") {
        localStorage.removeItem(STORAGE_KEY);
      }
    },
  };
}

export const recentFilesStore = createRecentFilesStore();
