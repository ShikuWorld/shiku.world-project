import * as esbuild from 'esbuild';
import * as fs from 'fs-extra';
esbuild
  .build({
    entryPoints: ['src/index.ts'],
    platform: 'node',
    bundle: true,
    external: ['better-sqlite3'],
    outfile: 'build/index.js'
  })
  .then(() => {
    fs.copySync('./public', './build/public', { overwrite: true });
  });
