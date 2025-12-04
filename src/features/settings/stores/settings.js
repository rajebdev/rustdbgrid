import { writable, get } from "svelte/store";
import {
  getSettings,
  saveSettings as saveSettingsApi,
  updateSetting as updateSettingApi,
} from "../../../core/integrations/tauri";

// Default settings
const defaultSettings = {
  theme: "auto",
  editor_font_size: 13,
  editor_line_numbers: true,
  editor_autocomplete: true,
  grid_page_size: 100,
  grid_row_numbers: true,
  grid_scroll_step: 24, // Scroll step in pixels (0 = use browser default)
  confirm_delete: true,
  auto_connect: false,
  last_connection_id: null,
};

// Create the settings store
function createSettingsStore() {
  const { subscribe, set, update } = writable(defaultSettings);

  return {
    subscribe,

    // Initialize settings from backend
    async init() {
      try {
        const settings = await getSettings();
        set(settings);
        return settings;
      } catch (error) {
        console.error("Failed to load settings:", error);
        // Use defaults if loading fails
        set(defaultSettings);
        return defaultSettings;
      }
    },

    // Save all settings
    async save(newSettings) {
      try {
        await saveSettingsApi(newSettings);
        set(newSettings);
        return true;
      } catch (error) {
        console.error("Failed to save settings:", error);
        return false;
      }
    },

    // Update a single setting
    async updateSetting(key, value) {
      try {
        const updatedSettings = await updateSettingApi(key, value);
        set(updatedSettings);
        return true;
      } catch (error) {
        console.error(`Failed to update setting ${key}:`, error);
        return false;
      }
    },

    // Get current settings synchronously
    get() {
      return get({ subscribe });
    },

    // Reset to defaults
    async reset() {
      try {
        await saveSettingsApi(defaultSettings);
        set(defaultSettings);
        return true;
      } catch (error) {
        console.error("Failed to reset settings:", error);
        return false;
      }
    },
  };
}

export const settings = createSettingsStore();
