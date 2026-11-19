use anyhow::Result;

use crate::commands::{install, tap};

pub fn run() -> Result<()> {
    tap::update_all()?;
    install::run("rustket", false)
}
