use anyhow::{Result, bail};

use crate::commands::{install, list, tap};

pub fn run() -> Result<()> {
    tap::update_all()?;
    let names = list::installed_names()?;
    if names.is_empty() {
        println!("no installed bits to upgrade");
        return Ok(());
    }

    let mut failures = Vec::new();
    for name in names {
        if let Err(error) = install::run(&name, false) {
            eprintln!("failed to upgrade {name}: {error:#}");
            failures.push(name);
        }
    }

    if !failures.is_empty() {
        bail!(
            "failed to upgrade {} bit(s): {}",
            failures.len(),
            failures.join(", ")
        );
    }

    println!("all installed bits upgraded");
    Ok(())
}
