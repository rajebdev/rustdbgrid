/**
 * Editor utility functions untuk handle execute selected text dan features lainnya
 */

/**
 * Get selected text atau full text jika tidak ada selection
 */
export function getSelectedOrFullText(editorView) {
  const selection = editorView.state.selection;

  if (selection.main.from === selection.main.to) {
    // No selection, return full text
    return {
      text: editorView.state.doc.toString(),
      isFullText: true,
      from: 0,
      to: editorView.state.doc.length,
    };
  }

  // Return selected text
  const selectedText = editorView.state.sliceDoc(
    selection.main.from,
    selection.main.to
  );
  return {
    text: selectedText,
    isFullText: false,
    from: selection.main.from,
    to: selection.main.to,
  };
}

/**
 * Get all statements (separated by semicolon)
 */
export function getAllStatements(text) {
  // Split by semicolon tapi exclude semicolon dalam strings
  const statements = [];
  let current = "";
  let inString = false;
  let stringChar = "";

  for (let i = 0; i < text.length; i++) {
    const char = text[i];
    const prevChar = i > 0 ? text[i - 1] : "";

    // Handle string quotes
    if ((char === "'" || char === '"') && prevChar !== "\\") {
      if (!inString) {
        inString = true;
        stringChar = char;
      } else if (char === stringChar) {
        inString = false;
        stringChar = "";
      }
    }

    // Split on semicolon if not in string
    if (char === ";" && !inString) {
      if (current.trim()) {
        statements.push(current.trim());
      }
      current = "";
    } else {
      current += char;
    }
  }

  // Add remaining statement
  if (current.trim()) {
    statements.push(current.trim());
  }

  return statements.filter((s) => s.length > 0);
}

/**
 * Execute selected text atau create new tab with results
 */
export async function executeSelectedText(
  editorView,
  selectedConn,
  selectedDb,
  executeInNewTab = false
) {
  if (!editorView || !selectedConn || !selectedDb) {
    return {
      success: false,
      message:
        "Missing connection or database. Please select connection and database first.",
    };
  }

  const { text, isFullText } = getSelectedOrFullText(editorView);

  if (!text.trim()) {
    return {
      success: false,
      message: "No text selected",
    };
  }

  return {
    success: true,
    query: text,
    isFullText,
    executeInNewTab,
    connectionId: selectedConn.id,
    databaseName: selectedDb,
  };
}
