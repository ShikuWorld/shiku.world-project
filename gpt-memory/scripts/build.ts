import * as esbuild from 'esbuild';
esbuild
  .build({
    entryPoints: ['src/index.ts'],
    platform: 'node',
    bundle: true,
    external: ['better-sqlite3'],
    outfile: 'build/index.js'
  })
  .then(() => {});
