// Critical CSS - load synchronously
import "bootstrap/dist/css/bootstrap.min.css";
import "./app.css";

// Bootstrap JS - load synchronously (needed for dropdowns, modals)
import "bootstrap/dist/js/bootstrap.bundle.min.js";

import App from "./App.svelte";
import { overrideConsole } from "./utils/logger.js";

// Override console BEFORE app loads
try {
  overrideConsole();
} catch (e) {
  console.error("Failed to load logger:", e);
}

// Lazy load FontAwesome CSS (non-blocking)
const loadFontAwesome = () => {
  import("@fortawesome/fontawesome-free/css/all.css");
};

// Load FontAwesome after app is interactive
if (document.readyState === "complete") {
  loadFontAwesome();
} else {
  window.addEventListener("load", loadFontAwesome, { once: true });
}

const app = new App({
  target: document.getElementById("app"),
});

export default app;
