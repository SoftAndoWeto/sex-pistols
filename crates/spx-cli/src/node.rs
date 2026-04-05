use std::path::Path;
use std::process::Command;

use crate::error::SpxError;

pub fn run(
    entry: &Path,
    args: &[String],
    cache_dir: &Path,
    loader_dir: &Path,
) -> Result<std::process::ExitStatus, SpxError> {
    let esm_loader = loader_dir.join("esm-loader.mjs");
    let cjs_register = loader_dir.join("cjs-register.cjs");

    let status = Command::new("node")
        .arg("--import")
        .arg(&esm_loader)
        .arg("--require")
        .arg(&cjs_register)
        .arg(entry)
        .args(args)
        .env("SPX_CACHE_DIR", cache_dir)
        .status()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SpxError::NodeNotFound
            } else {
                SpxError::Io(e)
            }
        })?;

    Ok(status)
}

pub fn loader_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("SPX_LOADER_DIR") {
        return std::path::PathBuf::from(dir);
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(bin_dir) = exe.parent() {
            let candidate = bin_dir.join("loader");
            if candidate.exists() {
                return candidate;
            }
            let candidate = bin_dir.join("../loader");
            if candidate.exists() {
                return candidate;
            }
        }
    }

    std::path::PathBuf::from("loader")
}
