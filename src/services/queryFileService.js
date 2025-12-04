/**
 * Query File Service - untuk mengelola penyimpanan query ke file lokal
 */
import { invoke } from "@tauri-apps/api/core";

/**
 * Load all query files from queries folder
 */
export async function loadQueriesFromFolder() {
  try {
    // Get config directory
    const configDir = await invoke("get_config_dir");
    const queriesDir = configDir + "/rustdbgrid/queries";

    // List all files in queries folder with content
    const queries = await invoke("list_query_files_with_content", {
      folderPath: queriesDir,
    });

    return queries;
  } catch (error) {
    console.error("Failed to load queries from folder:", error);
    return [];
  }
}

/**
 * Delete a query file
 */
export async function deleteQueryFile(filePath) {
  try {
    await invoke("delete_query_file", { filePath });
    return true;
  } catch (error) {
    console.error("Failed to delete query file:", error);
    throw error;
  }
}
