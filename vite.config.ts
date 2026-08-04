import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";

export default defineConfig({
  clearScreen: false,
  plugins: [vue()],
  resolve: {
    alias: {
      "@": "/src",
    },
  },
  server: {
    strictPort: true,
    port: 1421,
    host: "127.0.0.1",
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
