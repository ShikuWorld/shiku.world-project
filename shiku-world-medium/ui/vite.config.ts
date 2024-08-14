import { fileURLToPath, URL } from "node:url";

import vue from "@vitejs/plugin-vue";
import vuetify from "vite-plugin-vuetify";
import mkcert from "vite-plugin-mkcert";

/** @type {import('vite').UserConfig} */
export default {
  plugins: [vue(), vuetify({ autoImport: true }), mkcert()],
  server: {
    https: true,
    port: 8080,
  },
  base: "",
  resolve: {
    alias: {
      "@/editor": fileURLToPath(new URL("./src", import.meta.url)),
      "@/client": fileURLToPath(new URL("../client", import.meta.url)),
      "@/shared": fileURLToPath(new URL("../shared", import.meta.url)),
    },
  },
};
