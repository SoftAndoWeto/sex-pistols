# sex-pistols (spx)

> TypeScript execution tool for Node.js, written in Rust. Named after the Stand from JoJo's Bizarre Adventure Part 5.

A fast alternative to [tsx](https://github.com/privatenumber/tsx), powered by [OXC](https://oxc.rs/).

## How it works

`spx` pre-warms the entire dependency graph before Node.js starts:

1. OXC Resolver walks all imports in parallel (rayon)
2. OXC Transformer transpiles every `.ts`/`.tsx` file in parallel
3. Transpiled files are written to a tmpfs cache (keyed by blake3 hash)
4. Node.js starts with a small loader shim that reads from cache
5. IPC server handles cache misses (dynamic imports)

This means the Node.js loader hot path is just a filesystem read — no Rust round-trip per import.

## Usage

```sh
# Run a TypeScript file
spx run index.ts

# Watch mode — restart on file changes
spx watch index.ts

# Interactive REPL
spx repl
```

## Requirements

- Rust 1.80+
- Node.js 18+

## Building

```sh
cargo build --release
```
