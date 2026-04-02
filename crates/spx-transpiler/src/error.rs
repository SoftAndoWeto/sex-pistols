use thiserror::Error;

#[derive(Debug, Error)]
pub enum TranspileError {
    #[error("parse error in `{file}`: {message}")]
    Parse { file: String, message: String },

    #[error("transform error in `{file}`: {message}")]
    Transform { file: String, message: String },
}
