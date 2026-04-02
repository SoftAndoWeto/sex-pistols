pub mod error;
pub mod options;

pub use error::TranspileError;
pub use options::TranspileOptions;

use std::path::Path;

use oxc::allocator::Allocator;
use oxc::codegen::{Codegen, CodegenOptions};
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::transformer::{TransformOptions, Transformer};

#[cfg(test)]
mod tests;

/// Output produced by [`transpile`].
pub struct TranspileOutput {
    pub code: String,
    /// Serialised source map JSON, present only when `opts.source_maps` is `true`.
    pub map: Option<String>,
}

/// Transpile `source` according to `opts`.
///
/// Strips TypeScript types so Node.js can execute the result without further tooling.
pub fn transpile(source: &str, opts: &TranspileOptions) -> Result<TranspileOutput, TranspileError> {
    let allocator = Allocator::default();
    let source_type = opts.source_type();
    let path = Path::new(&opts.filename);

    // --- Parse ---
    let parser_ret = Parser::new(&allocator, source, source_type).parse();

    if !parser_ret.errors.is_empty() {
        let message = fmt_errors(&parser_ret.errors);
        return Err(TranspileError::Parse {
            file: opts.filename.clone(),
            message,
        });
    }

    let mut program = parser_ret.program;

    // --- Semantic analysis (required by the transformer) ---
    let scoping = SemanticBuilder::new()
        .build(&program)
        .semantic
        .into_scoping();

    // --- Transform (strip TS types) ---
    let transform_options = TransformOptions::default();
    let transformer_ret =
        Transformer::new(&allocator, path, &transform_options)
            .build_with_scoping(scoping, &mut program);

    if !transformer_ret.errors.is_empty() {
        let message = fmt_errors(&transformer_ret.errors);
        return Err(TranspileError::Transform {
            file: opts.filename.clone(),
            message,
        });
    }

    // --- Codegen ---
    let codegen_options = CodegenOptions {
        source_map_path: opts.source_maps.then(|| path.to_path_buf()),
        ..CodegenOptions::default()
    };
    let codegen_ret = Codegen::new().with_options(codegen_options).build(&program);

    let map = codegen_ret.map.map(|sm| sm.to_json_string());

    Ok(TranspileOutput {
        code: codegen_ret.code,
        map,
    })
}

fn fmt_errors(errors: &[oxc::diagnostics::OxcDiagnostic]) -> String {
    errors
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("; ")
}
