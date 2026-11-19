use std::fs;
use std::io::ErrorKind;

use anyhow::{Context, Result};

use crate::paths;

pub fn run() -> Result<()> {
    let bin_dir = paths::root().join("bin");
    let entries = match fs::read_dir(&bin_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read {}", bin_dir.display()));
        }
    };
    let mut names = entries
        .map(|entry| {
            entry
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .with_context(|| format!("failed to read entry in {}", bin_dir.display()))
        })
        .collect::<Result<Vec<_>>>()?;
    names.sort();
    for name in names {
        println!("{name}");
    }
    Ok(())
}
