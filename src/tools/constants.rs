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

// Aurora SDK ssh key path
#[cfg(unix)]
pub const SSH_EMULATOR_KEY_PATH: &str = "~/AuroraOS/vmshare/ssh/private_keys/sdk";

#[cfg(windows)]
pub const SSH_EMULATOR_KEY_PATH: &str = "C:\\AuroraOS\\vmshare\\ssh\\private_keys\\sdk";

// Search name image Build Tools
pub const BT_IMAGES_SEARCH: &'static [&'static str] = &[r"^(aurora).+(build).+\:.+", r"^(hub).+(build).+\:.+"];
