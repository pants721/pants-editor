use std::{os::unix::fs::PermissionsExt, path::{Path, PathBuf}};

use anyhow::{anyhow, Result};
use home::home_dir;

// SOURCE: https://www.reddit.com/r/rust/comments/leewn4/how_to_check_if_a_file_is_executable/
pub fn is_executable(path: &Path) -> Result<bool> {
    let permissions = path.metadata()?.permissions();

    Ok(permissions.mode() & 0o111 != 0)
}

pub fn pe_config_file_path() -> Result<PathBuf> {
    let home = home_dir().ok_or_else(|| anyhow!("Failed to get home directory"))?;
    let path = home.join(".config").join("pe").join("config.toml");

    Ok(path)
}
