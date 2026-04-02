use std::path::Path;

use oxc::span::SourceType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranspileOptions {
    /// Source file path — used to infer the source type and in error messages / source maps.
    pub filename: String,
    /// Emit an inline source map alongside the generated code.
    #[serde(default)]
    pub source_maps: bool,
}

impl Default for TranspileOptions {
    fn default() -> Self {
        Self {
            filename: String::from("<unknown>"),
            source_maps: false,
        }
    }
}

impl TranspileOptions {
    pub fn new(filename: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            ..Self::default()
        }
    }

    /// Infer [`SourceType`] from the file extension.
    /// Falls back to ESM JavaScript for unknown extensions.
    pub(crate) fn source_type(&self) -> SourceType {
        SourceType::from_path(Path::new(&self.filename)).unwrap_or_else(|_| SourceType::mjs())
    }
}
