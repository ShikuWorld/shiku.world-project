import { defineConfig } from 'vite';
import { viteSingleFile } from 'vite-plugin-singlefile';

export default defineConfig({
  plugins: [viteSingleFile()],
  build: {
    rollupOptions: {
      input: 'src/main.ts',
      output: {
        file: 'dist/index.js',
        format: 'iife',
        name: 'MyApp',
      },
    },
  },
});
