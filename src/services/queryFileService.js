/**
 * Query File Service - untuk mengelola penyimpanan query ke file lokal
 */

export async function saveQueryToFile(query, filename) {
  try {
    // Membuat blob dari query
    const blob = new Blob([query], { type: "text/plain" });
    const url = URL.createObjectURL(blob);

    // Membuat link download dan trigger download
    const a = document.createElement("a");
    a.href = url;
    a.download = filename || `query_${Date.now()}.sql`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    return { success: true };
  } catch (error) {
    console.error("Failed to save query to file:", error);
    return { success: false, error };
  }
}

export async function loadQueryFromFile(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();

    reader.onload = (event) => {
      try {
        const content = event.target.result;
        resolve({ success: true, content });
      } catch (error) {
        reject({ success: false, error });
      }
    };

    reader.onerror = () => {
      reject({ success: false, error: "Failed to read file" });
    };

    reader.readAsText(file);
  });
}

export function downloadQueries(queries, filename = "queries.json") {
  try {
    const data = JSON.stringify(queries, null, 2);
    const blob = new Blob([data], { type: "application/json" });
    const url = URL.createObjectURL(blob);

    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    return { success: true };
  } catch (error) {
    console.error("Failed to download queries:", error);
    return { success: false, error };
  }
}

/**
 * Load all query files from queries folder
 */
export async function loadQueriesFromFolder() {
  const { invoke } = await import("@tauri-apps/api/core");

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
  const { invoke } = await import("@tauri-apps/api/core");

  try {
    await invoke("delete_query_file", { filePath });
    return true;
  } catch (error) {
    console.error("Failed to delete query file:", error);
    throw error;
  }
}
