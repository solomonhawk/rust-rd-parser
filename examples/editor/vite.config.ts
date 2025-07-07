import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { comlink } from "vite-plugin-comlink";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), comlink()],
  worker: {
    plugins: () => [comlink()],
  },
  server: {
    fs: {
      strict: false,
    },
  },
});
