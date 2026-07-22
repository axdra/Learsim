import { defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [tailwindcss()],
  clearScreen: false,
  server: {
    port: 1421,
    strictPort: true,
  },
  build: {
    target: "es2021",
    sourcemap: false,
  },
});
