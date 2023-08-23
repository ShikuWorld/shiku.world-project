import * as esbuild from 'esbuild';
import * as fs from 'fs-extra';
esbuild
  .build({
    entryPoints: ['src/index.ts'],
    platform: 'node',
    bundle: true,
    outfile: 'build/index.js'
  })
  .then(() => {
    fs.copySync('./public', './build/public', { overwrite: true });
    fs.copySync('./index.html', './build/index.html', {
      overwrite: true
    });
  });
