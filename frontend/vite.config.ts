import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: "es2022",
    sourcemap: true,
  },
  test: {
    environment: "happy-dom",
    include: ["tests/**/*.test.ts"],
    // Svelte 5 ships browser-only and server-only runtimes; happy-dom needs
    // the browser one to render components and run lifecycle hooks.
    server: {
      deps: {
        inline: [/svelte/],
      },
    },
  },
  resolve: {
    conditions: ["browser"],
  },
});
