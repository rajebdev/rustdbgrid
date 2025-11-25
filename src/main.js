import "bootstrap/dist/css/bootstrap.min.css";
import "bootstrap/dist/js/bootstrap.bundle.min.js";
import "./app.css";
import App from "./App.svelte";
import { attachConsole } from "@tauri-apps/plugin-log";

// Attach console to Tauri's log plugin to show all console.log in terminal
if (window.__TAURI__) {
  attachConsole();
}

const app = new App({
  target: document.getElementById("app"),
});

export default app;
