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
