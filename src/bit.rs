use std::collections::HashMap;
use std::fs::{self, DirEntry};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::paths;

#[derive(Deserialize)]
pub struct Bit {
    pub name: String,
    pub version: String,
    pub platform: HashMap<String, Artifact>,
}

#[derive(Deserialize)]
pub struct Artifact {
    pub url: String,
    pub sha256: String,
    pub bin: String,
}

pub fn load(name: &str) -> Result<Bit> {
    let matching_files = files()?
        .into_iter()
        .filter(|path| path.file_stem().is_some_and(|stem| stem == name))
        .collect::<Vec<_>>();
    if matching_files.is_empty() {
        bail!("no bit: {name}");
    }
    if matching_files.len() > 1 {
        let locations = matching_files
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        bail!("multiple tapped bits named {name}: {locations}");
    }

    let path = &matching_files[0];
    let contents =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&contents).with_context(|| format!("failed to parse {}", path.display()))
}

pub fn names() -> Result<Vec<String>> {
    let mut names = files()?
        .into_iter()
        .filter_map(|path| {
            path.file_stem()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    Ok(names)
}

fn files() -> Result<Vec<PathBuf>> {
    let tap_root = paths::taps();
    let owners = match fs::read_dir(&tap_root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read {}", tap_root.display()));
        }
    };

    let mut files = Vec::new();
    for owner in owners {
        let owner = owner.with_context(|| format!("failed to read {}", tap_root.display()))?;
        if !owner
            .file_type()
            .with_context(|| format!("failed to inspect {}", owner.path().display()))?
            .is_dir()
        {
            continue;
        }
        for tap in read_dirs(&owner)? {
            if tap
                .file_type()
                .with_context(|| format!("failed to inspect {}", tap.path().display()))?
                .is_dir()
            {
                files.extend(bit_files_in(&tap.path().join("bits"))?);
            }
        }
    }
    Ok(files)
}

fn read_dirs(entry: &DirEntry) -> Result<Vec<DirEntry>> {
    let path = entry.path();
    fs::read_dir(&path)
        .with_context(|| format!("failed to read {}", path.display()))?
        .map(|entry| entry.with_context(|| format!("failed to read {}", path.display())))
        .collect()
}

fn bit_files_in(directory: &Path) -> Result<Vec<PathBuf>> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read {}", directory.display()));
        }
    };
    entries
        .filter_map(|entry| match entry {
            Ok(entry) => match entry.path().extension().is_some_and(|ext| ext == "toml") {
                true => Some(Ok(entry.path())),
                false => None,
            },
            Err(error) => Some(Err(error)),
        })
        .map(|path| path.with_context(|| format!("failed to read {}", directory.display())))
        .collect()
}
