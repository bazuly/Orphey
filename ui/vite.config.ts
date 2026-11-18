import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";

function stripCrossorigin(): Plugin {
  return {
    name: "strip-crossorigin",
    transformIndexHtml(html) {
      return html.replace(/\s+crossorigin(?:="[^"]*")?/g, "");
    },
  };
}

export default defineConfig({
  base: "./",
  plugins: [react(), stripCrossorigin()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "es2021",
    minify: !process.env.TAURI_ENV_DEBUG,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});