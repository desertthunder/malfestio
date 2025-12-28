import tailwindcss from "@tailwindcss/vite";
import solid from "vite-plugin-solid";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [solid(), tailwindcss()],
  server: { proxy: { "/api": { target: "http://localhost:8080", changeOrigin: true } } },
  test: {
    environment: "jsdom",
    ui: false,
    watch: false,
    server: { deps: { inline: ["@solidjs/router", "solid-js"] } },
  },
});
