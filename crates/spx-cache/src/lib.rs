pub mod error;
pub mod key;

pub use error::CacheError;
pub use key::CacheKey;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use dashmap::DashMap;
use memmap2::Mmap;

#[cfg(test)]
mod tests;

/// Two-level transpile cache.
///
/// - **L1** – `DashMap` in process memory (zero-copy lookup via `Arc<str>`).
/// - **L2** – directory on disk (tmpfs recommended); each entry is a `.js` file
///   named by its `CacheKey` hex.
pub struct Cache {
    l1: DashMap<CacheKey, Arc<str>>,
    l2_dir: PathBuf,
}

impl Cache {
    /// Create a cache backed by `l2_dir`. The directory is created if needed.
    pub fn new(l2_dir: PathBuf) -> Result<Self, CacheError> {
        fs::create_dir_all(&l2_dir).map_err(|e| CacheError::Io {
            path: l2_dir.to_string_lossy().into_owned(),
            source: e,
        })?;
        Ok(Self {
            l1: DashMap::new(),
            l2_dir,
        })
    }

    /// Create a cache in a fresh subdirectory under the OS temp dir.
    pub fn with_temp_dir() -> Result<Self, CacheError> {
        let dir = std::env::temp_dir().join(format!(
            "spx-cache-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        Self::new(dir)
    }

    /// The L2 directory path.
    pub fn l2_dir(&self) -> &Path {
        &self.l2_dir
    }

    /// Look up transpiled JS for `path` by reading its current mtime.
    ///
    /// Returns `None` on a cache miss — the caller must transpile and [`put`](Self::put).
    pub fn get(&self, path: &Path) -> Result<Option<Arc<str>>, CacheError> {
        let key = self.key_for_path(path)?;
        Ok(self.get_by_key(key))
    }

    /// Store `js` as the transpile output for `path` at its current mtime.
    pub fn put(&self, path: &Path, js: &str) -> Result<CacheKey, CacheError> {
        let key = self.key_for_path(path)?;
        self.put_by_key(key, js)?;
        Ok(key)
    }

    /// Get by a pre-computed key. Promotes an L2 hit into L1.
    pub fn get_by_key(&self, key: CacheKey) -> Option<Arc<str>> {
        if let Some(v) = self.l1.get(&key) {
            return Some(v.clone());
        }
        self.read_l2(key)
    }

    /// Store `js` under `key` in both L1 and L2.
    pub fn put_by_key(&self, key: CacheKey, js: &str) -> Result<(), CacheError> {
        self.l1.insert(key, Arc::from(js));
        self.write_l2(key, js)
    }

    fn key_for_path(&self, path: &Path) -> Result<CacheKey, CacheError> {
        let meta = fs::metadata(path).map_err(|e| CacheError::Metadata {
            path: path.to_string_lossy().into_owned(),
            source: e,
        })?;
        let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        Ok(CacheKey::compute(path, mtime))
    }

    fn l2_path(&self, key: CacheKey) -> PathBuf {
        self.l2_dir.join(format!("{}.js", key.to_hex()))
    }

    fn read_l2(&self, key: CacheKey) -> Option<Arc<str>> {
        let path = self.l2_path(key);
        let file = fs::File::open(&path).ok()?;
        // SAFETY: L2 files are content-addressed and written once, so they
        // are never mutated while a mapping is live.
        let mmap = unsafe { Mmap::map(&file).ok()? };
        let s = std::str::from_utf8(&mmap).ok()?;
        let arc: Arc<str> = Arc::from(s);
        self.l1.insert(key, arc.clone()); // promote to L1
        Some(arc)
    }

    fn write_l2(&self, key: CacheKey, js: &str) -> Result<(), CacheError> {
        let path = self.l2_path(key);
        fs::write(&path, js.as_bytes()).map_err(|e| CacheError::Io {
            path: path.to_string_lossy().into_owned(),
            source: e,
        })
    }
}
