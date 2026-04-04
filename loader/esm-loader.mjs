import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join, extname } from 'node:path';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const { SpxEnvError, SpxManifestError, SpxCacheMissError } = require('./errors.cjs');

const TS_RE = /\.(ts|mts|cts)$/;
const JS_TO_TS = { '.js': '.ts', '.mjs': '.mts', '.cjs': '.cts' };

const cacheDir = process.env.SPX_CACHE_DIR;
if (!cacheDir) throw new SpxEnvError('SPX_CACHE_DIR');

let manifest;
try {
  manifest = JSON.parse(readFileSync(join(cacheDir, 'manifest.json'), 'utf8'));
} catch (err) {
  throw new SpxManifestError(join(cacheDir, 'manifest.json'), err);
}

function normalise(p) {
  return p.replace(/\\/g, '/');
}

export async function resolve(specifier, context, nextResolve) {
  const ext = extname(specifier);
  const tsExt = JS_TO_TS[ext];
  if (tsExt) {
    try {
      return await nextResolve(specifier.slice(0, -ext.length) + tsExt, context);
    } catch {}
  }
  return nextResolve(specifier, context);
}

export async function load(url, context, nextLoad) {
  if (!TS_RE.test(url)) return nextLoad(url, context);

  const filePath = fileURLToPath(url);
  const key = manifest[normalise(filePath)] ?? manifest[filePath];
  if (!key) throw new SpxCacheMissError(filePath);

  const source = readFileSync(join(cacheDir, key + '.js'), 'utf8');
  const format = url.endsWith('.cts') ? 'commonjs' : 'module';
  return { format, source, shortCircuit: true };
}
