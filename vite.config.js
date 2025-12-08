import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import monacoEditorPlugin from "vite-plugin-monaco-editor";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [
    svelte(),
    monacoEditorPlugin.default({
      languageWorkers: ["editorWorkerService", "json"],
    }),
  ],
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
    chunkSizeWarningLimit: 1000,
    rollupOptions: {
      output: {
        manualChunks: {
          // Monaco Editor - separate chunk (very large)
          monaco: ["monaco-editor"],
          // Tauri APIs - separate chunk
          "tauri-api": [
            "@tauri-apps/api/core",
            "@tauri-apps/api/event",
            "@tauri-apps/api/window",
            "@tauri-apps/api/path",
          ],
          "tauri-plugins": [
            "@tauri-apps/plugin-dialog",
            "@tauri-apps/plugin-fs",
            "@tauri-apps/plugin-shell",
          ],
          // Svelte runtime
          svelte: ["svelte", "svelte/store"],
        },
      },
    },
  },
  // Pre-bundle heavy dependencies for faster dev startup
  optimizeDeps: {
    include: [
      "bootstrap/dist/js/bootstrap.bundle.min.js",
      "monaco-editor",
      "@tauri-apps/api",
      "svelte",
      "svelte/store",
    ],
    // Exclude FontAwesome from pre-bundle (lazy loaded)
    exclude: ["@fortawesome/fontawesome-free"],
  },
});
