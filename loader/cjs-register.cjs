'use strict';

const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const Module = require('node:module');
const { SpxEnvError, SpxManifestError, SpxCacheMissError } = require('./errors.cjs');

const cacheDir = process.env.SPX_CACHE_DIR;
if (!cacheDir) throw new SpxEnvError('SPX_CACHE_DIR');

const manifestPath = join(cacheDir, 'manifest.json');
let manifest;
try {
  manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
} catch (err) {
  throw new SpxManifestError(manifestPath, err);
}

function normalise(p) {
  return p.replace(/\\/g, '/');
}

function loadTs(mod, filename) {
  const key = manifest[normalise(filename)] ?? manifest[filename];
  if (!key) throw new SpxCacheMissError(filename);
  const source = readFileSync(join(cacheDir, key + '.js'), 'utf8');
  mod._compile(source, filename);
}

Module._extensions['.ts'] = loadTs;
Module._extensions['.mts'] = loadTs;
Module._extensions['.cts'] = loadTs;
