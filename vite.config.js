import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    host: host || false,
    port: 1420,
    strictPort: true,
    hmr: host ? { protocol: "ws", host, port: 1430 } : undefined,
    // Faster cold start
    warmup: {
      clientFiles: ["./src/main.js", "./src/App.svelte"],
    },
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: ["es2021", "chrome100", "safari13"],
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
  // Pre-bundle heavy dependencies for faster dev startup
  optimizeDeps: {
    include: [
      "bootstrap/dist/js/bootstrap.bundle.min.js",
      "codemirror",
      "@codemirror/state",
      "@codemirror/view",
      "@codemirror/lang-sql",
      "@codemirror/autocomplete",
      "@codemirror/commands",
      "@codemirror/theme-one-dark",
      "@tauri-apps/api",
      "svelte",
      "svelte/store",
    ],
    // Exclude FontAwesome from pre-bundle (lazy loaded)
    exclude: ["@fortawesome/fontawesome-free"],
  },
});
