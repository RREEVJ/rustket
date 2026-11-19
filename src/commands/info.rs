use anyhow::Result;

use crate::bit;

pub fn run(name: &str) -> Result<()> {
    let b = bit::load(name)?;
    let mut platforms = b.platform.keys().collect::<Vec<_>>();
    platforms.sort();

    println!("name: {}", b.name);
    println!("version: {}", b.version);
    println!("platforms:");
    for platform in platforms {
        println!("  {platform}");
    }
    Ok(())
}
