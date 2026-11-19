use std::fs;
use std::io::ErrorKind;

use anyhow::{Context, Result};

use crate::paths;

pub fn run(name: &str) -> Result<()> {
    let root = paths::root();
    let link = root.join("bin").join(name);
    match fs::symlink_metadata(&link) {
        Ok(_) => fs::remove_file(&link)
            .with_context(|| format!("failed to remove {}", link.display()))?,
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| format!("failed to inspect {}", link.display()));
        }
    }

    let cellar = root.join("cellar").join(name);
    match fs::remove_dir_all(&cellar) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| format!("failed to remove {}", cellar.display()));
        }
    }
    println!("uninstalled {name}");
    Ok(())
}
