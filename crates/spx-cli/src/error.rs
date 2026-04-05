use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum SpxError {
    #[error(transparent)]
    Transpile(#[from] spx_transpiler::TranspileError),

    #[error(transparent)]
    Cache(#[from] spx_cache::CacheError),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

#[error("json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("node.js not found — is it installed and on PATH?")]
    NodeNotFound,
}
