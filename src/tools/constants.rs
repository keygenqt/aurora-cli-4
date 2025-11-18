// Global sysroot dir
#[cfg(unix)]
pub const SYSROOT_GLOBAL_DIR_PATH: &str = "~/.aurora-sysroot";
#[cfg(windows)]
pub const SYSROOT_GLOBAL_DIR_PATH: &str = "~\\.aurora-sysroot";

// Name sysroot unix dir
pub const SYSROOT_UNIX_DIR_NAME: &str = ".aurora-sysroot-unix";

// Create prefix container for sysroot
pub const CONTAINER_SYS_PREFIX: &str = "aurora-builder-sys";

// Create prefix container for other
pub const CONTAINER_PREFIX: &str = "aurora-builder";

// Search name image Build Tools
pub const BT_IMAGE_PREFIX: &str = "aurora-build-tools";
/// Debug log
#[cfg(debug_assertions)]
pub const PRINT_DEBUG: bool = false;

#[cfg(not(debug_assertions))]
pub const PRINT_DEBUG: bool = false;
