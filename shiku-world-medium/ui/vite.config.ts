import { fileURLToPath, URL } from "node:url";

import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import vuetify from "vite-plugin-vuetify";
import mkcert from "vite-plugin-mkcert";
export default defineConfig({
  plugins: [vue(), vuetify({ autoImport: true }), mkcert()],
  server: {
    https: true,
    port: 8080,
  },
  resolve: {
    alias: {
      "@/editor": fileURLToPath(new URL("./src", import.meta.url)),
      "@/client": fileURLToPath(new URL("../client", import.meta.url)),
    },
  },
});
