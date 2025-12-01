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
