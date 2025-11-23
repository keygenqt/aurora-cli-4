// Global sysroot dir
#[cfg(unix)]
pub const SYSROOT_GLOBAL_DIR_PATH: &str = "~/.aurora-sysroot";

#[cfg(windows)]
pub const SYSROOT_GLOBAL_DIR_PATH: &str = "~\\.aurora-sysroot";

// Name sysroot unix dir
pub const SYSROOT_UNIX_DIR_NAME: &str = ".aurora-sysroot-unix";

// Create prefix container for sysroot
pub const CONTAINER_PREFIX_SYS: &str = "aurora-builder-sys";

// Create prefix container for other
pub const CONTAINER_PREFIX: &str = "aurora-builder";

// Search name image Build Tools
pub const BT_IMAGES_SEARCH: &'static [&'static str] = &[r"^(aurora).+(tools)\:.+", r"^(hub).+(tools)\:.+"];
