import "@fortawesome/fontawesome-free/css/all.css";
import "bootstrap/dist/css/bootstrap.min.css";
import "bootstrap/dist/js/bootstrap.bundle.min.js";
import "./app.css";
import App from "./App.svelte";
import { overrideConsole } from "./utils/logger.js";

// Override console BEFORE app loads
try {
  overrideConsole();
} catch (e) {
  console.error("Failed to load logger:", e);
}

const app = new App({
  target: document.getElementById("app"),
});

export default app;
