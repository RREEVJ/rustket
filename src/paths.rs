use std::path::PathBuf;

pub fn root() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .expect("no home dir")
        .join(".rustket")
}

pub fn taps() -> PathBuf {
    root().join("taps")
}

pub fn current_platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}
