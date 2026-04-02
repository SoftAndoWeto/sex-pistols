use std::path::Path;
use std::time::{Duration, SystemTime};

use super::{Cache, CacheKey};

fn fixed_mtime(secs: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
}

fn fresh_tmp_path(suffix: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "spx-test-{}-{}.{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        suffix
    ))
}

#[test]
fn key_same_inputs_equal() {
    let path = Path::new("/tmp/foo.ts");
    let mtime = fixed_mtime(1_700_000_000);
    assert_eq!(CacheKey::compute(path, mtime), CacheKey::compute(path, mtime));
}

#[test]
fn key_different_path_differs() {
    let mtime = fixed_mtime(1_700_000_000);
    assert_ne!(
        CacheKey::compute(Path::new("/tmp/a.ts"), mtime),
        CacheKey::compute(Path::new("/tmp/b.ts"), mtime),
    );
}

#[test]
fn key_different_mtime_differs() {
    let path = Path::new("/tmp/foo.ts");
    assert_ne!(
        CacheKey::compute(path, fixed_mtime(1_000)),
        CacheKey::compute(path, fixed_mtime(2_000)),
    );
}

#[test]
fn miss_returns_none() {
    let cache = Cache::with_temp_dir().unwrap();
    let key = CacheKey::compute(Path::new("/nonexistent.ts"), fixed_mtime(0));
    assert!(cache.get_by_key(key).is_none());
}

#[test]
fn l1_hit() {
    let cache = Cache::with_temp_dir().unwrap();
    let key = CacheKey::compute(Path::new("/tmp/a.ts"), fixed_mtime(1_000));

    cache.put_by_key(key, "const x = 1;").unwrap();
    assert_eq!(&*cache.get_by_key(key).unwrap(), "const x = 1;");
}

#[test]
fn l2_hit_after_l1_cleared() {
    let cache = Cache::with_temp_dir().unwrap();
    let key = CacheKey::compute(Path::new("/tmp/b.ts"), fixed_mtime(42));

    cache.put_by_key(key, "let y = 2;").unwrap();

    let cache2 = Cache::new(cache.l2_dir().to_path_buf()).unwrap();
    assert_eq!(&*cache2.get_by_key(key).unwrap(), "let y = 2;");
}

#[test]
fn put_and_get_by_path() {
    let cache = Cache::with_temp_dir().unwrap();

    let tmp = fresh_tmp_path("ts");
    std::fs::write(&tmp, "const z: number = 3;").unwrap();

    cache.put(&tmp, "const z = 3;").unwrap();
    let result = cache.get(&tmp).unwrap().unwrap();
    assert_eq!(&*result, "const z = 3;");

    std::fs::remove_file(&tmp).ok();
}

#[test]
fn get_missing_file_returns_error() {
    let cache = Cache::with_temp_dir().unwrap();
    assert!(cache.get(Path::new("/no/such/file.ts")).is_err());
}
