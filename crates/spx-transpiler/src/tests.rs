use crate::{TranspileOptions, transpile};

fn opts(filename: &str) -> TranspileOptions {
    TranspileOptions::new(filename)
}

#[test]
fn strips_type_annotation() {
    let src = "const x: number = 42;";
    let out = transpile(src, &opts("index.ts")).unwrap();
    assert!(!out.code.contains(": number"), "type annotation should be stripped");
    assert!(out.code.contains("42"));
}

#[test]
fn strips_interface() {
    let src = "interface Foo { bar: string } export {};";
    let out = transpile(src, &opts("foo.ts")).unwrap();
    assert!(!out.code.contains("interface"));
}

#[test]
fn plain_js_passthrough() {
    let src = "const hello = 'world';";
    let out = transpile(src, &opts("hello.mjs")).unwrap();
    assert!(out.code.contains("hello"));
    assert!(out.code.contains("world"));
}

#[test]
fn source_map_absent_by_default() {
    let src = "const x: number = 1;";
    let out = transpile(src, &opts("x.ts")).unwrap();
    assert!(out.map.is_none());
}

#[test]
fn source_map_present_when_requested() {
    let src = "const x: number = 1;";
    let mut o = opts("x.ts");
    o.source_maps = true;
    let out = transpile(src, &o).unwrap();
    assert!(out.map.is_some());
}

#[test]
fn parse_error_is_reported() {
    let src = "const = ;";
    let result = transpile(src, &opts("bad.ts"));
    assert!(result.is_err());
}
