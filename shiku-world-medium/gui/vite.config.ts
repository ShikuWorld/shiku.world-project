import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import vuetify from "vite-plugin-vuetify";
import path from "path";

export default defineConfig({
  build: { minify: false },
  plugins: [vue(), vuetify({ autoImport: true })],
  resolve: {
    alias: [{ find: "@", replacement: path.resolve(__dirname, "src") }],
  },
  base: "./",
});
