use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use oxc::allocator::Allocator;
use oxc::ast::ast::Statement;
use oxc::parser::Parser;
use oxc::span::SourceType;
use oxc_resolver::{ResolveOptions, Resolver};
use rayon::prelude::*;
use spx_cache::{Cache, CacheKey};
use spx_transpiler::{TranspileOptions, transpile};

use crate::error::SpxError;

const TS_EXTENSIONS: &[&str] = &[".ts", ".mts", ".cts", ".js", ".mjs", ".cjs"];

pub struct GraphOutput {
    pub cache: Cache,
}

pub fn build(entry: &Path) -> Result<GraphOutput, SpxError> {
    let entry = entry.canonicalize()?;
    let files = collect_files(&entry)?;

    let cache = Cache::with_temp_dir()?;

    let pairs: Vec<(PathBuf, CacheKey)> = files
        .par_iter()
        .map(|path| transpile_file(&cache, path).map(|key| (path.clone(), key)))
        .collect::<Result<Vec<_>, SpxError>>()?;

    write_manifest(cache.l2_dir(), &pairs)?;

    Ok(GraphOutput { cache })
}

pub(crate) fn collect_files(entry: &Path) -> Result<Vec<PathBuf>, SpxError> {
    let resolver = Resolver::new(ResolveOptions {
        extensions: TS_EXTENSIONS.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    });

    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut queue: VecDeque<PathBuf> = VecDeque::new();

    queue.push_back(entry.to_path_buf());

    while let Some(path) = queue.pop_front() {
        if !visited.insert(path.clone()) {
            continue;
        }

        let specifiers = extract_imports(&path)?;
        let dir = path.parent().unwrap_or(Path::new("."));

        for spec in specifiers {
            match resolver.resolve(dir, &spec) {
                Ok(resolved) => {
                    let resolved_path = resolved.into_path_buf();
                    if !visited.contains(&resolved_path) {
                        queue.push_back(resolved_path);
                    }
                }
                Err(_) => {
                    // external package or unresolvable — skip
                }
            }
        }
    }

    Ok(visited.into_iter().collect())
}

pub(crate) fn extract_imports(path: &Path) -> Result<Vec<String>, SpxError> {
    let source = std::fs::read_to_string(path)?;
    let source_type = SourceType::from_path(path).unwrap_or_default();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, source_type).parse();

    let mut specifiers = Vec::new();
    for stmt in &ret.program.body {
        match stmt {
            Statement::ImportDeclaration(decl) => {
                specifiers.push(decl.source.value.as_str().to_owned());
            }
            Statement::ExportNamedDeclaration(decl) => {
                if let Some(src) = &decl.source {
                    specifiers.push(src.value.as_str().to_owned());
                }
            }
            Statement::ExportAllDeclaration(decl) => {
                specifiers.push(decl.source.value.as_str().to_owned());
            }
            _ => {}
        }
    }

    Ok(specifiers)
}

fn transpile_file(cache: &Cache, path: &Path) -> Result<CacheKey, SpxError> {
    let source = std::fs::read_to_string(path)?;
    let opts = TranspileOptions {
        filename: path.to_string_lossy().into_owned(),
        source_maps: false,
    };
    let result = transpile(&source, &opts)?;
    let key = cache.put(path, &result.code)?;
    Ok(key)
}

fn write_manifest(cache_dir: &Path, pairs: &[(PathBuf, CacheKey)]) -> Result<(), SpxError> {
    let map: HashMap<String, String> = pairs
        .iter()
        .map(|(path, key)| {
            let normalised = path.to_string_lossy().replace('\\', "/");
            (normalised, key.to_hex())
        })
        .collect();

    let manifest_path = cache_dir.join("manifest.json");
    std::fs::write(manifest_path, serde_json::to_string(&map)?)?;
    Ok(())
}
