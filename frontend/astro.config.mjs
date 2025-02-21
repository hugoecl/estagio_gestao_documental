// @ts-check
import { defineConfig } from "astro/config";
import tailwindcss from "@tailwindcss/vite";

import svelte from "@astrojs/svelte";

import node from "@astrojs/node";

// https://astro.build/config
export default defineConfig({
  vite: {
    plugins: [tailwindcss()],
  },

  integrations: [svelte()],

  prefetch: {
    prefetchAll: true,
    defaultStrategy: "viewport",
  },

  output: "server",
  adapter: node({
    mode: "standalone",
  }),
});
