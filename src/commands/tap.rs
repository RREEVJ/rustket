use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as Proc;

use anyhow::{Context, Result, anyhow, bail};

use crate::cli::TapCommand;
use crate::paths;

const DEFAULT_TAP: &str = "Rustket/rustket-bits";
const DEFAULT_TAP_OWNER: &str = "Rustket";
const DEFAULT_TAP_REPOSITORY: &str = "rustket-bits";

pub fn run(command: TapCommand) -> Result<()> {
    match command {
        TapCommand::Add { repository } => add(&repository),
        TapCommand::List => list(),
        TapCommand::Remove { repository } => remove(&repository),
        TapCommand::Update => update_all(),
    }
}

pub fn ensure_default() -> Result<()> {
    ensure_default_at(&paths::taps(), || add(DEFAULT_TAP))
}

pub fn update_all() -> Result<()> {
    ensure_default()?;
    let taps = tap_paths()?;
    for (name, path) in taps {
        let output = Proc::new("git")
            .arg("-C")
            .arg(&path)
            .args(["pull", "--ff-only"])
            .output()
            .with_context(|| format!("failed to update tap {name}"))?;
        if !output.status.success() {
            bail!(
                "failed to update tap {name}: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        println!("updated {name}");
    }
    Ok(())
}

fn add(repository: &str) -> Result<()> {
    let (name, url, destination) = tap_location(repository)?;
    if destination.exists() {
        bail!("tap already exists: {name}");
    }
    let parent = destination
        .parent()
        .ok_or_else(|| anyhow!("invalid tap path for {name}"))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    let output = Proc::new("git")
        .args(["clone", "--depth", "1", "--"])
        .arg(&url)
        .arg(&destination)
        .output()
        .with_context(|| format!("failed to clone tap {name}"))?;
    if !output.status.success() {
        bail!(
            "failed to clone tap {name}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    println!("added tap {name}");
    Ok(())
}

fn list() -> Result<()> {
    ensure_default()?;
    let taps = tap_paths()?;
    for (name, _) in taps {
        println!("{name}");
    }
    Ok(())
}

fn remove(repository: &str) -> Result<()> {
    let (name, _, destination) = tap_location(repository)?;
    let metadata = match fs::symlink_metadata(&destination) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bail!("tap is not installed: {name}");
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to inspect {}", destination.display()));
        }
    };
    if !metadata.file_type().is_dir() {
        bail!("tap path is not a directory: {}", destination.display());
    }
    fs::remove_dir_all(&destination).with_context(|| format!("failed to remove tap {name}"))?;
    println!("removed tap {name}");
    Ok(())
}

fn tap_paths() -> Result<Vec<(String, PathBuf)>> {
    let tap_root = paths::taps();
    let owners = match fs::read_dir(&tap_root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read {}", tap_root.display()));
        }
    };

    let mut taps = Vec::new();
    for owner in owners {
        let owner = owner.with_context(|| format!("failed to read {}", tap_root.display()))?;
        if !owner
            .file_type()
            .with_context(|| format!("failed to inspect {}", owner.path().display()))?
            .is_dir()
        {
            continue;
        }
        let owner_name = owner.file_name().to_string_lossy().into_owned();
        let owner_path = owner.path();
        for repository in fs::read_dir(&owner_path)
            .with_context(|| format!("failed to read {}", owner_path.display()))?
        {
            let repository =
                repository.with_context(|| format!("failed to read {}", owner_path.display()))?;
            if !repository
                .file_type()
                .with_context(|| format!("failed to inspect {}", repository.path().display()))?
                .is_dir()
            {
                continue;
            }
            let repo_name = repository.file_name().to_string_lossy().into_owned();
            taps.push((format!("{owner_name}/{repo_name}"), repository.path()));
        }
    }
    taps.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(taps)
}

fn ensure_default_at(tap_root: &Path, add_default: impl FnOnce() -> Result<()>) -> Result<()> {
    let path = tap_root
        .join(DEFAULT_TAP_OWNER)
        .join(DEFAULT_TAP_REPOSITORY);
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_dir() => Ok(()),
        Ok(_) => bail!(
            "default tap path exists but is not a directory: {}",
            path.display()
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => add_default(),
        Err(error) => Err(error).with_context(|| format!("failed to inspect {}", path.display())),
    }
}

fn tap_location(repository: &str) -> Result<(String, String, PathBuf)> {
    let repository = repository
        .strip_prefix("https://github.com/")
        .unwrap_or(repository);
    let repository = repository.trim_end_matches('/');
    let repository = repository.strip_suffix(".git").unwrap_or(repository);
    let mut parts = repository.split('/');
    let owner = parts.next().unwrap_or_default();
    let repo = parts.next().unwrap_or_default();
    if parts.next().is_some() || !valid_component(owner) || !valid_component(repo) {
        bail!("expected a GitHub tap as owner/repo or https://github.com/owner/repo");
    }

    let name = format!("{owner}/{repo}");
    let url = format!("https://github.com/{owner}/{repo}.git");
    let destination = paths::taps().join(owner).join(repo);
    Ok((name, url, destination))
}

fn valid_component(component: &str) -> bool {
    !component.is_empty()
        && component != "."
        && component != ".."
        && component
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{ensure_default_at, tap_location};

    fn temp_dir() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("rustket-tap-test-{}-{nonce}", std::process::id()))
    }

    #[test]
    fn accepts_github_url_and_shorthand() {
        let (url_name, url, _) = tap_location("https://github.com/Rustket/rustket-bits").unwrap();
        let (short_name, short_url, _) = tap_location("Rustket/rustket-bits").unwrap();

        assert_eq!(url_name, "Rustket/rustket-bits");
        assert_eq!(url, "https://github.com/Rustket/rustket-bits.git");
        assert_eq!(short_name, url_name);
        assert_eq!(short_url, url);
    }

    #[test]
    fn rejects_path_traversal_and_non_github_urls() {
        assert!(tap_location("../outside/repo").is_err());
        assert!(tap_location("git@github.com:Rustket/rustket-bits.git").is_err());
    }

    #[test]
    fn ensures_default_tap_only_when_missing() {
        let root = temp_dir();
        let mut added = false;
        ensure_default_at(&root, || {
            added = true;
            Ok(())
        })
        .unwrap();
        assert!(added);

        let existing_root = root.join("existing");
        fs::create_dir_all(existing_root.join("Rustket").join("rustket-bits")).unwrap();
        let mut added = false;
        ensure_default_at(&existing_root, || {
            added = true;
            Ok(())
        })
        .unwrap();
        assert!(!added);

        let conflicting_root = root.join("conflicting");
        let conflicting_path = conflicting_root.join("Rustket").join("rustket-bits");
        fs::create_dir_all(conflicting_path.parent().unwrap()).unwrap();
        fs::write(&conflicting_path, "not a directory").unwrap();
        assert!(ensure_default_at(&conflicting_root, || Ok(())).is_err());

        fs::remove_dir_all(root).unwrap();
    }
}
