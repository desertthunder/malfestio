import tailwindcss from "@tailwindcss/vite";
import path from "path";
import { loadEnv } from "vite";
import solid from "vite-plugin-solid";
import { defineConfig } from "vitest/config";

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  const appUrl = env.APP_URL || "http://localhost:3000";
  let host = "localhost";
  try {
    const url = new URL(appUrl);
    host = url.hostname;
  } catch {
    console.warn("Invalid APP_URL in .env, defaulting to localhost");
  }

  return {
    plugins: [solid(), tailwindcss()],
    resolve: {
      alias: {
        $lib: path.resolve(__dirname, "src/lib"),
        $pages: path.resolve(__dirname, "src/pages"),
        $components: path.resolve(__dirname, "src/components"),
        $ui: path.resolve(__dirname, "src/components/ui"),
      },
    },
    server: {
      host: "0.0.0.0",
      allowedHosts: [host, "localhost", "127.0.0.1", ".ts.net", ".ngrok-free.app"],
      proxy: { "/api": { target: "http://localhost:8080", changeOrigin: true } },
      port: 3000,
    },
    test: {
      environment: "jsdom",
      ui: false,
      watch: false,
      server: { deps: { inline: [/@solidjs/, /solid-js/, /solid-motionone/, /motion/] } },
    },
  };
});
