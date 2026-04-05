use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::graph;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(path)
}

#[test]
fn extract_imports_static() {
    let mut specs = graph::extract_imports(&fixture("imports/entry.ts")).unwrap();
    specs.sort();
    assert_eq!(specs, vec!["./bar", "./baz", "./foo", "./qux"]);
}

#[test]
fn extract_imports_no_imports() {
    let specs = graph::extract_imports(&fixture("standalone/index.ts")).unwrap();
    assert!(specs.is_empty());
}

#[test]
fn collect_files_single() {
    let files = graph::collect_files(&fixture("single/index.ts")).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("index.ts"));
}

#[test]
fn collect_files_two_level_graph() {
    let mut files = graph::collect_files(&fixture("graph/a.ts")).unwrap();
    files.sort();
    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|p| p.ends_with("a.ts")));
    assert!(files.iter().any(|p| p.ends_with("b.ts")));
}

#[test]
fn build_writes_manifest() {
    let output = graph::build(&fixture("build/main.ts")).unwrap();
    let manifest_path = output.cache.l2_dir().join("manifest.json");

    assert!(manifest_path.exists());

    let map: HashMap<String, String> =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();

    assert_eq!(map.len(), 1);
    let key = map.values().next().unwrap();
    assert!(output.cache.l2_dir().join(format!("{key}.js")).exists());
}

#[test]
fn manifest_paths_use_forward_slashes() {
    let output = graph::build(&fixture("build/main.ts")).unwrap();
    let map: HashMap<String, String> = serde_json::from_str(
        &fs::read_to_string(output.cache.l2_dir().join("manifest.json")).unwrap(),
    )
    .unwrap();

    for key in map.keys() {
        assert!(!key.contains('\\'), "path contains backslash: {key}");
    }
}
