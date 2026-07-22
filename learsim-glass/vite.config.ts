import { defineConfig } from "vite";

// Vite config tuned for Tauri: fixed dev port, no auto-open, and don't clear
// the terminal so Rust build output stays visible.
export default defineConfig({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: "es2021",
    // webkit2gtk on the Pi is older; avoid overly-modern minified output.
    minify: "esbuild",
    sourcemap: false,
  },
});
