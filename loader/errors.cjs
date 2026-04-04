'use strict';

class SpxEnvError extends Error {
  constructor(name) {
    super(`spx: environment variable ${name} is not set`);
    this.name = 'SpxEnvError';
  }
}

class SpxManifestError extends Error {
  constructor(path, cause) {
    super(`spx: failed to read manifest at ${path}: ${cause.message}`);
    this.name = 'SpxManifestError';
    this.cause = cause;
  }
}

class SpxCacheMissError extends Error {
  constructor(filePath) {
    super(`spx: ${filePath} was not pre-warmed — run via \`spx run\``);
    this.name = 'SpxCacheMissError';
  }
}

module.exports = { SpxEnvError, SpxManifestError, SpxCacheMissError };
