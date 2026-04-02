use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("I/O error for {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read metadata for {path}: {source}")]
    Metadata {
        path: String,
        #[source]
        source: std::io::Error,
    },
}
