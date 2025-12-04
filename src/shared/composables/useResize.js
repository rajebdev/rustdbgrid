import { onMount } from "svelte";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";

/**
 * Hook for managing window resize and maximize/unmaximize events
 */
export function useWindowResize(onMaximizeChange) {
  onMount(async () => {
    const appWindow = getCurrentWindow();
    let isMaximized = await appWindow.isMaximized();

    // Notify initial state
    if (onMaximizeChange) {
      onMaximizeChange(isMaximized);
    }

    const unlisten = await listen("tauri://resize", async () => {
      const maximized = await appWindow.isMaximized();
      if (maximized !== isMaximized) {
        isMaximized = maximized;
        if (onMaximizeChange) {
          onMaximizeChange(maximized);
        }
      }
    });

    return () => {
      unlisten();
    };
  });
}

/**
 * Hook for managing sidebar resize
 */
export function useSidebarResize(options = {}) {
  const { minWidth = 200, maxWidth = 600, onResize } = options;

  function handleMouseMove(event, isResizing) {
    if (!isResizing) return null;

    const newWidth = event.clientX;
    if (newWidth >= minWidth && newWidth <= maxWidth) {
      return newWidth;
    }
    return null;
  }

  return {
    handleMouseMove,
  };
}

/**
 * Hook for managing editor resize
 */
export function useEditorResize(options = {}) {
  const { minHeight = 150, maxHeight = 700 } = options;

  onMount(() => {
    let isResizing = false;
    let currentHeight = options.initialHeight || 300;

    const handleStartResize = (e) => {
      isResizing = true;
      e.detail.event.preventDefault();
      if (options.onResizeStart) {
        options.onResizeStart();
      }
    };

    const handleResizeMove = (e) => {
      if (!isResizing) return;
      const event = e.detail.event;
      const mainContent = document.querySelector(".main-content-area");
      if (mainContent) {
        const rect = mainContent.getBoundingClientRect();
        const newHeight = event.clientY - rect.top;
        if (newHeight >= minHeight && newHeight <= maxHeight) {
          currentHeight = newHeight;
          if (options.onResize) {
            options.onResize(newHeight);
          }
        }
      }
    };

    const handleResizeEnd = () => {
      if (isResizing) {
        isResizing = false;
        if (options.onResizeEnd) {
          options.onResizeEnd();
        }
      }
    };

    window.addEventListener("start-editor-resize", handleStartResize);
    window.addEventListener("editor-resize-move", handleResizeMove);
    window.addEventListener("mouseup", handleResizeEnd);

    return () => {
      window.removeEventListener("start-editor-resize", handleStartResize);
      window.removeEventListener("editor-resize-move", handleResizeMove);
      window.removeEventListener("mouseup", handleResizeEnd);
    };
  });
}
