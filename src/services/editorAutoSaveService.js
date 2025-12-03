import { invoke } from "@tauri-apps/api/core";
import { saveAutoQuery } from "../utils/tauri";

/**
 * Setup auto-save with debouncing
 */
export function setupAutoSave(tabId, getContent, config, delay = 2000) {
  let autoSaveTimeout;

  const saveFunction = async () => {
    const content = getContent();
    if (!content || !content.trim()) return;

    try {
      const { connId, dbName, filePath } = config;

      // Save to backend file system (.autosave.json)
      await saveAutoQuery(tabId, content, connId, dbName);

      // Also save to the actual file if tab has a filePath
      if (filePath) {
        const configDir = await invoke("get_config_dir");

        // Build full absolute path
        let fullPath = filePath;
        if (!fullPath.match(/^[a-zA-Z]:[\\\\/]/) && !fullPath.startsWith("/")) {
          // Relative path, make it absolute
          const sep = navigator.platform.toLowerCase().includes("win")
            ? "\\"
            : "/";
          fullPath =
            configDir + sep + "rustdbgrid" + sep + filePath.replace(/\//g, sep);
        }

        await invoke("auto_save_query_file", {
          filePath: fullPath,
          content,
        });
      }
    } catch (error) {
      console.error("Failed to auto-save query:", error);
    }
  };

  const trigger = () => {
    // Clear previous timeout
    if (autoSaveTimeout) {
      clearTimeout(autoSaveTimeout);
    }

    // Debounce auto-save
    autoSaveTimeout = setTimeout(saveFunction, delay);
  };

  const cleanup = () => {
    if (autoSaveTimeout) {
      clearTimeout(autoSaveTimeout);
    }
  };

  return {
    trigger,
    cleanup,
    saveNow: saveFunction,
  };
}
