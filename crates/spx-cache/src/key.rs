use std::path::Path;
use std::time::SystemTime;

/// A content-addressed cache key derived from a file's path and mtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheKey(blake3::Hash);

impl CacheKey {
    /// Compute a key from a file path and its last-modified time.
    pub fn compute(path: &Path, mtime: SystemTime) -> Self {
        let secs = mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut h = blake3::Hasher::new();
        h.update(path.to_string_lossy().as_bytes());
        h.update(&secs.to_le_bytes());
        CacheKey(h.finalize())
    }

    /// Hex string, used as the L2 filename.
    pub fn to_hex(self) -> String {
        self.0.to_hex().to_string()
    }
}
