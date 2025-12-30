import tailwindcss from "@tailwindcss/vite";
import path from "path";
import solid from "vite-plugin-solid";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [solid(), tailwindcss()],
  resolve: {
    alias: {
      $lib: path.resolve(__dirname, "src/lib"),
      $pages: path.resolve(__dirname, "src/pages"),
      $components: path.resolve(__dirname, "src/components"),
      $ui: path.resolve(__dirname, "src/components/ui"),
    },
  },
  server: { proxy: { "/api": { target: "http://localhost:8080", changeOrigin: true } } },
  test: {
    environment: "jsdom",
    ui: false,
    watch: false,
    server: { deps: { inline: [/@solidjs/, /solid-js/, /solid-motionone/, /motion/] } },
  },
});
