import { onMount } from "svelte";

/**
 * Setup keyboard shortcuts for the application
 * @param {Object} handlers - Object containing handler functions
 */
export function useKeyboardShortcuts(handlers) {
  onMount(() => {
    const handleKeyDown = (event) => {
      // Ctrl/Cmd + N: New Query
      if (
        (event.ctrlKey || event.metaKey) &&
        event.key === "n" &&
        !event.shiftKey
      ) {
        event.preventDefault();
        handlers.newQuery?.();
      }
      // Ctrl/Cmd + O: Open File
      else if ((event.ctrlKey || event.metaKey) && event.key === "o") {
        event.preventDefault();
        handlers.openFile?.();
      }
      // Ctrl/Cmd + S: Save Query
      else if (
        (event.ctrlKey || event.metaKey) &&
        event.key === "s" &&
        !event.shiftKey
      ) {
        event.preventDefault();
        handlers.saveQuery?.();
      }
      // Ctrl/Cmd + Shift + S: Save As
      else if (
        (event.ctrlKey || event.metaKey) &&
        event.shiftKey &&
        event.key === "S"
      ) {
        event.preventDefault();
        handlers.saveAs?.();
      }
      // Ctrl/Cmd + B: Toggle Sidebar
      else if ((event.ctrlKey || event.metaKey) && event.key === "b") {
        event.preventDefault();
        handlers.toggleSidebar?.();
      }
      // Ctrl/Cmd + Shift + C: New Connection
      else if (
        (event.ctrlKey || event.metaKey) &&
        event.shiftKey &&
        event.key === "C"
      ) {
        event.preventDefault();
        handlers.newConnection?.();
      }
      // F5 or Ctrl/Cmd + Enter: Execute Query
      else if (
        // event.key === "F5" ||
        (event.ctrlKey || event.metaKey) &&
        event.key === "Enter"
      ) {
        event.preventDefault();
        handlers.execute?.();
      }
      // Ctrl/Cmd + Shift + Enter: Execute Script
      else if (
        (event.ctrlKey || event.metaKey) &&
        event.shiftKey &&
        event.key === "Enter"
      ) {
        event.preventDefault();
        handlers.executeScript?.();
      }
      // F5 with Shift: Refresh
      else if (event.shiftKey && event.key === "F5") {
        event.preventDefault();
        handlers.refresh?.();
      }
      // Ctrl/Cmd + W: Close Tab
      else if ((event.ctrlKey || event.metaKey) && event.key === "w") {
        event.preventDefault();
        handlers.closeTab?.();
      }
      // Ctrl/Cmd + Tab: Next Tab
      else if (
        (event.ctrlKey || event.metaKey) &&
        event.key === "Tab" &&
        !event.shiftKey
      ) {
        event.preventDefault();
        handlers.nextTab?.();
      }
      // Ctrl/Cmd + Shift + Tab: Previous Tab
      else if (
        (event.ctrlKey || event.metaKey) &&
        event.shiftKey &&
        event.key === "Tab"
      ) {
        event.preventDefault();
        handlers.previousTab?.();
      }
      // Ctrl/Cmd + Shift + T: Toggle Theme
      else if (
        (event.ctrlKey || event.metaKey) &&
        event.shiftKey &&
        event.key === "T"
      ) {
        event.preventDefault();
        handlers.toggleTheme?.();
      }
      // Ctrl/Cmd + K then Ctrl/Cmd + S: Keyboard Shortcuts
      else if ((event.ctrlKey || event.metaKey) && event.key === "k") {
        event.preventDefault();
        const waitForSecondKey = (e) => {
          if ((e.ctrlKey || e.metaKey) && e.key === "s") {
            e.preventDefault();
            handlers.showKeyboardShortcuts?.();
            window.removeEventListener("keydown", waitForSecondKey);
          }
          setTimeout(() => {
            window.removeEventListener("keydown", waitForSecondKey);
          }, 1000);
        };
        window.addEventListener("keydown", waitForSecondKey);
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  });
}
