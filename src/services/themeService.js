import { get } from "svelte/store";
import {
  themePreference,
  activeTheme,
  initializeThemeListener,
  applyTheme,
} from "../stores/theme";

let unsubscribeTheme = null;
let unsubscribeListener = null;

/**
 * Initialize the theme system
 * Should be called once during app initialization
 */
export async function initializeTheme() {
  console.log("[FRONTEND] Initializing theme...");
  // Initialize theme preference from backend settings
  await themePreference.init();
  console.log("[FRONTEND] Theme preference initialized");

  // Set up system preference listener
  unsubscribeListener = initializeThemeListener();

  // Apply initial theme
  const currentTheme = get(activeTheme);
  applyTheme(currentTheme);

  // Subscribe to theme changes and apply them
  unsubscribeTheme = activeTheme.subscribe((theme) => {
    applyTheme(theme);
    // Dispatch custom event for components that need to react to theme changes
    if (typeof window !== "undefined") {
      window.dispatchEvent(
        new CustomEvent("themechange", { detail: { theme } })
      );
    }
  });

  return () => {
    if (unsubscribeTheme) unsubscribeTheme();
    if (unsubscribeListener) unsubscribeListener();
  };
}

/**
 * Get the current active theme
 */
export function getCurrentTheme() {
  return get(activeTheme);
}

/**
 * Get the current theme preference
 */
export function getThemePreference() {
  return get(themePreference);
}

/**
 * Set theme to light mode
 */
export function setLightTheme() {
  themePreference.setLight();
}

/**
 * Set theme to dark mode
 */
export function setDarkTheme() {
  themePreference.setDark();
}

/**
 * Set theme to auto (follow system)
 */
export function setAutoTheme() {
  themePreference.setAuto();
}

/**
 * Toggle between themes: light -> dark -> auto -> light
 */
export function toggleTheme() {
  themePreference.toggle();
}

/**
 * Get CodeMirror theme configuration based on current theme
 */
export function getEditorTheme(theme = null) {
  const currentTheme = theme || get(activeTheme);

  if (currentTheme === "dark") {
    return {
      "&": {
        backgroundColor: "var(--editor-bg)",
        color: "var(--editor-text)",
        height: "100%",
      },
      ".cm-content": {
        fontFamily: "'Consolas', 'Monaco', 'Courier New', monospace",
        fontSize: "13px",
        padding: "8px 0",
        caretColor: "var(--editor-cursor)",
      },
      ".cm-line": {
        padding: "0 8px",
      },
      ".cm-gutters": {
        backgroundColor: "var(--editor-gutter-bg)",
        color: "var(--editor-gutter-text)",
        border: "none",
        borderRight: "1px solid var(--border-color)",
      },
      ".cm-activeLineGutter": {
        backgroundColor: "var(--editor-line-active)",
      },
      ".cm-activeLine": {
        backgroundColor: "var(--editor-line-active)",
      },
      ".cm-selectionLayer": {
        zIndex: "-1 !important",
        pointerEvents: "none !important",
      },
      ".cm-selectionBackground": {
        backgroundColor: "var(--editor-selection) !important",
      },
      "&.cm-focused .cm-selectionBackground": {
        backgroundColor: "var(--editor-selection) !important",
      },
      "::selection": {
        backgroundColor: "var(--editor-selection)",
      },
      ".cm-line, .cm-line *": {
        userSelect: "text !important",
        WebkitUserSelect: "text !important",
      },
      ".cm-cursor, .cm-dropCursor": {
        borderLeftColor: "var(--editor-cursor)",
        borderLeftWidth: "2px",
      },
      ".cm-tooltip-autocomplete": {
        backgroundColor: "var(--bg-dropdown)",
        border: "1px solid var(--border-color)",
        boxShadow: "var(--shadow-dropdown)",
      },
      ".cm-tooltip-autocomplete > ul > li": {
        color: "var(--text-primary)",
      },
      ".cm-tooltip-autocomplete > ul > li[aria-selected]": {
        backgroundColor: "var(--accent-blue)",
        color: "var(--text-inverse)",
      },
      ".cm-completionIcon": {
        color: "var(--text-secondary)",
      },
      ".cm-completionLabel": {
        color: "var(--text-primary)",
      },
      ".cm-completionDetail": {
        color: "var(--text-muted)",
      },
    };
  }

  // Light theme (default)
  return {
    "&": {
      backgroundColor: "var(--editor-bg)",
      color: "var(--editor-text)",
      height: "100%",
    },
    ".cm-content": {
      fontFamily: "'Consolas', 'Monaco', 'Courier New', monospace",
      fontSize: "13px",
      padding: "8px 0",
      caretColor: "var(--editor-cursor)",
    },
    ".cm-line": {
      padding: "0 8px",
    },
    ".cm-gutters": {
      backgroundColor: "var(--editor-gutter-bg)",
      color: "var(--editor-gutter-text)",
      border: "none",
      borderRight: "1px solid var(--border-color)",
    },
    ".cm-activeLineGutter": {
      backgroundColor: "var(--editor-line-active)",
    },
    ".cm-activeLine": {
      backgroundColor: "var(--editor-line-active)",
    },
    ".cm-selectionLayer": {
      zIndex: "-1 !important",
      pointerEvents: "none !important",
    },
    ".cm-selectionBackground": {
      backgroundColor: "var(--editor-selection) !important",
    },
    "&.cm-focused .cm-selectionBackground": {
      backgroundColor: "var(--editor-selection) !important",
    },
    "::selection": {
      backgroundColor: "var(--editor-selection)",
    },
    ".cm-line, .cm-line *": {
      userSelect: "text !important",
      WebkitUserSelect: "text !important",
    },
    ".cm-cursor, .cm-dropCursor": {
      borderLeftColor: "var(--editor-cursor)",
      borderLeftWidth: "2px",
    },
    ".cm-tooltip-autocomplete": {
      backgroundColor: "var(--bg-dropdown)",
      border: "1px solid var(--border-color)",
      boxShadow: "var(--shadow-dropdown)",
    },
    ".cm-tooltip-autocomplete > ul > li": {
      color: "var(--text-primary)",
    },
    ".cm-tooltip-autocomplete > ul > li[aria-selected]": {
      backgroundColor: "var(--accent-blue)",
      color: "var(--text-inverse)",
    },
    ".cm-completionIcon": {
      color: "var(--text-secondary)",
    },
    ".cm-completionLabel": {
      color: "var(--text-primary)",
    },
    ".cm-completionDetail": {
      color: "var(--text-muted)",
    },
  };
}
