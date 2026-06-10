use crate::error::{AppError, AppResult};
use std::path::{Component, Path, PathBuf};

/// Canonicalize an absolute path string (rejects `..` components in relative paths).
pub fn canonicalize_path(path: &str) -> AppResult<PathBuf> {
    let target = Path::new(path);
    if target.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(AppError::Validation(
            "Path must not contain parent directory references".to_string(),
        ));
    }
    target
        .canonicalize()
        .map_err(|e| AppError::Validation(format!("Invalid path: {}", e)))
}

/// Ensure `path` resolves under `root` (prevents directory traversal).
pub fn ensure_within_root(root: &str, path: &str) -> AppResult<PathBuf> {
    let root = Path::new(root);
    let target = Path::new(path);

    let root_canon = root
        .canonicalize()
        .map_err(|e| AppError::Validation(format!("Invalid root path: {}", e)))?;

    let resolved = if target.is_absolute() {
        target.to_path_buf()
    } else {
        root_canon.join(target)
    };

    let mut normalized = PathBuf::new();
    for component in resolved.components() {
        match component {
            Component::ParentDir => {
                return Err(AppError::Validation(
                    "Path escapes vault root".to_string(),
                ));
            }
            Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }

    if !normalized.starts_with(&root_canon) {
        return Err(AppError::Validation(
            "Path must be within vault root".to_string(),
        ));
    }

    Ok(normalized)
}
