import { writable, derived, get } from "svelte/store";
import { getTheme, setTheme as setThemeApi } from "../utils/tauri";

// Theme modes: 'light', 'dark', 'auto'

function getSystemPreference() {
  if (typeof window === "undefined") return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function createThemeStore() {
  const { subscribe, set, update } = writable("auto");
  let initialized = false;

  return {
    subscribe,

    // Initialize from backend
    async init() {
      if (initialized) return get({ subscribe });

      try {
        const theme = await getTheme();
        set(theme);
        initialized = true;
        return theme;
      } catch (error) {
        console.error("Failed to load theme from settings:", error);
        // Fallback to auto
        set("auto");
        return "auto";
      }
    },

    // Set theme and save to backend
    set: async (value) => {
      set(value);
      try {
        await setThemeApi(value);
      } catch (error) {
        console.error("Failed to save theme:", error);
      }
    },

    setLight: async () => {
      set("light");
      try {
        await setThemeApi("light");
      } catch (error) {
        console.error("Failed to save theme:", error);
      }
    },

    setDark: async () => {
      set("dark");
      try {
        await setThemeApi("dark");
      } catch (error) {
        console.error("Failed to save theme:", error);
      }
    },

    setAuto: async () => {
      set("auto");
      try {
        await setThemeApi("auto");
      } catch (error) {
        console.error("Failed to save theme:", error);
      }
    },

    toggle: () => {
      update((current) => {
        let newTheme;
        if (current === "light") newTheme = "dark";
        else if (current === "dark") newTheme = "auto";
        else newTheme = "light";

        // Save async, don't await
        setThemeApi(newTheme).catch((err) =>
          console.error("Failed to save theme:", err)
        );

        return newTheme;
      });
    },
  };
}

// Store for the user's preference (light, dark, or auto)
export const themePreference = createThemeStore();

// Store for system preference changes
export const systemPreference = writable(getSystemPreference());

// Derived store for the actual theme being applied
export const activeTheme = derived(
  [themePreference, systemPreference],
  ([$themePreference, $systemPreference]) => {
    if ($themePreference === "auto") {
      return $systemPreference;
    }
    return $themePreference;
  }
);

// Initialize system preference listener
export function initializeThemeListener() {
  if (typeof window === "undefined") return;

  const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

  const handleChange = (e) => {
    systemPreference.set(e.matches ? "dark" : "light");
  };

  // Modern browsers
  if (mediaQuery.addEventListener) {
    mediaQuery.addEventListener("change", handleChange);
  } else {
    // Legacy browsers
    mediaQuery.addListener(handleChange);
  }

  return () => {
    if (mediaQuery.removeEventListener) {
      mediaQuery.removeEventListener("change", handleChange);
    } else {
      mediaQuery.removeListener(handleChange);
    }
  };
}

// Apply theme to document with loading overlay
export function applyTheme(theme) {
  if (typeof document === "undefined") return;

  // Create and show loading overlay
  let overlay = document.getElementById("theme-transition-overlay");
  if (!overlay) {
    overlay = document.createElement("div");
    overlay.id = "theme-transition-overlay";
    overlay.style.cssText = `
      position: fixed;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      background: ${theme === "dark" ? "#1e1e1e" : "#f5f5f5"};
      z-index: 99999;
      opacity: 1;
      transition: opacity 0.15s ease-out;
      pointer-events: none;
    `;
    document.body.appendChild(overlay);
  } else {
    overlay.style.background = theme === "dark" ? "#1e1e1e" : "#f5f5f5";
    overlay.style.opacity = "1";
  }

  // Apply theme
  document.documentElement.setAttribute("data-theme", theme);

  // Also update meta theme-color for mobile browsers
  const metaThemeColor = document.querySelector('meta[name="theme-color"]');
  if (metaThemeColor) {
    metaThemeColor.setAttribute(
      "content",
      theme === "dark" ? "#1e1e1e" : "#f5f5f5"
    );
  }

  // Fade out and remove overlay
  requestAnimationFrame(() => {
    setTimeout(() => {
      overlay.style.opacity = "0";
      setTimeout(() => {
        if (overlay.parentNode) {
          overlay.parentNode.removeChild(overlay);
        }
      }, 150);
    }, 50);
  });
}
