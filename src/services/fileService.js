import { message, open, save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";

// Helper function for showing messages
export async function showMessage(msg, title = "RustDBGrid") {
  try {
    await message(msg, { title, kind: "info" });
  } catch (error) {
    console.error("Dialog error:", error);
    alert(msg);
  }
}

export async function showError(msg, title = "Error") {
  try {
    await message(msg, { title, kind: "error" });
  } catch (error) {
    console.error("Dialog error:", error);
    alert(msg);
  }
}

/**
 * File Service - Handles file operations
 */
export const fileService = {
  /**
   * Open SQL file
   */
  async openFile() {
    const filePath = await open({
      title: "Open SQL File",
      filters: [
        {
          name: "SQL Files",
          extensions: ["sql", "txt"],
        },
      ],
    });

    if (filePath) {
      const content = await readTextFile(filePath);
      const fileName = filePath.split(/[\\/]/).pop();
      return {
        name: fileName,
        path: filePath,
        content,
      };
    }
    return null;
  },

  /**
   * Save query to file
   */
  async saveQuery(filePath, content) {
    await writeTextFile(filePath, content);
  },

  /**
   * Save query as new file
   */
  async saveQueryAs(defaultFileName, content) {
    const filePath = await save({
      title: "Save SQL File",
      filters: [
        {
          name: "SQL Files",
          extensions: ["sql"],
        },
      ],
      defaultPath: `${defaultFileName}.sql`,
    });

    if (filePath) {
      await writeTextFile(filePath, content);
      const fileName = filePath.split(/[\\/]/).pop().replace(".sql", "");
      return {
        path: filePath,
        name: fileName,
      };
    }
    return null;
  },

  /**
   * Export data to file
   */
  async exportData(data, format = null) {
    const filters = [
      { name: "CSV Files", extensions: ["csv"] },
      { name: "JSON Files", extensions: ["json"] },
      { name: "SQL Insert", extensions: ["sql"] },
    ];

    const filePath = await save({
      title: "Export Data",
      filters,
    });

    if (filePath) {
      const ext = format || filePath.split(".").pop().toLowerCase();
      await invoke("export_data", {
        data,
        filePath,
        format: ext,
      });
      return true;
    }
    return false;
  },

  /**
   * Import data from file
   */
  async importData() {
    const file = await open({
      title: "Import Data",
      filters: [
        { name: "CSV Files", extensions: ["csv"] },
        { name: "JSON Files", extensions: ["json"] },
        { name: "SQL Files", extensions: ["sql"] },
      ],
    });

    if (file) {
      // TODO: Implement import logic
      return file;
    }
    return null;
  },

  /**
   * Auto-save query to queries folder
   */
  async autoSaveQuery(fileName, content) {
    try {
      // Get config directory and create queries folder path
      const configDir = await invoke("get_config_dir");

      // Use proper path separator for current OS
      const queriesDir = configDir + "/rustdbgrid/queries";

      // Create full file path with proper separator
      const filePath = queriesDir + "/" + fileName;

      // Invoke Rust command to auto-save
      const result = await invoke("auto_save_query_file", {
        filePath,
        content,
      });

      return {
        path: result.path || filePath,
        name: fileName,
      };
    } catch (error) {
      console.error("Failed to auto-save query:", error);
      throw error;
    }
  },
};
