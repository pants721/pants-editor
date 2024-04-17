use std::{os::unix::fs::PermissionsExt, path::Path};

use anyhow::Result;

// SOURCE: https://www.reddit.com/r/rust/comments/leewn4/how_to_check_if_a_file_is_executable/
pub fn is_executable(path: &Path) -> Result<bool> {
    let permissions = path.metadata()?.permissions();

    Ok(permissions.mode() & 0o111 != 0)
}
