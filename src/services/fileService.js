import { message } from "@tauri-apps/plugin-dialog";

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
    const { open } = await import("@tauri-apps/plugin-dialog");
    const file = await open({
      title: "Open SQL File",
      filters: [
        {
          name: "SQL Files",
          extensions: ["sql", "txt"],
        },
      ],
    });

    if (file) {
      const { readTextFile } = await import("@tauri-apps/plugin-fs");
      const content = await readTextFile(file.path);
      return {
        name: file.name,
        path: file.path,
        content,
      };
    }
    return null;
  },

  /**
   * Save query to file
   */
  async saveQuery(filePath, content) {
    const { writeTextFile } = await import("@tauri-apps/plugin-fs");
    await writeTextFile(filePath, content);
  },

  /**
   * Save query as new file
   */
  async saveQueryAs(defaultFileName, content) {
    const { save } = await import("@tauri-apps/plugin-dialog");
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
      const { writeTextFile } = await import("@tauri-apps/plugin-fs");
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
    const { save } = await import("@tauri-apps/plugin-dialog");
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
      const { invoke } = await import("@tauri-apps/api/core");
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
    const { open } = await import("@tauri-apps/plugin-dialog");
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
};
