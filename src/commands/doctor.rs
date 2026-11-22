use std::fs;
use std::path::Path;
use std::process::Command as Proc;

use anyhow::{Result, anyhow};

use crate::paths;

pub fn run() -> Result<()> {
    let root = paths::root();
    let taps = paths::taps();
    let bin = root.join("bin");
    let mut failures = 0;

    println!("platform: {}", paths::current_platform());
    failures += usize::from(!check("~/.rustket exists", root.is_dir()));
    failures += usize::from(!check("~/.rustket/taps exists", taps.is_dir()));

 let on_path = std::env::var_os("PATH").is_some_and(|p| {
    std::env::split_paths(&p).any(|e| e == bin || e.canonicalize().ok() == bin.canonicalize().ok())
});

    let git_installed = Proc::new("git")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success());
    failures += usize::from(!check("git is installed", git_installed));

    let no_broken_links = !has_broken_symlink(&bin)?;
    failures += usize::from(!check("no broken symlinks in bin", no_broken_links));

    if failures > 0 {
        return Err(anyhow!("{failures} doctor check(s) failed"));
    }
    println!("all good");
    Ok(())
}

fn check(label: &str, passed: bool) -> bool {
    println!("{} - {label}", if passed { "ok" } else { "FAIL" });
    passed
}

fn has_broken_symlink(bin: &Path) -> Result<bool> {
    let entries = match fs::read_dir(bin) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    for entry in entries {
        let entry = entry?;
        let link = entry.path();
        if fs::symlink_metadata(&link)?.file_type().is_symlink() {
            let target = fs::read_link(&link)?;
            let target = if target.is_absolute() {
                target
            } else {
                link.parent().unwrap_or(bin).join(target)
            };
            if !target.exists() {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
