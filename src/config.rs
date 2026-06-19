use std::{path::PathBuf, sync::LazyLock};

/// System-wide configuration directory used by Rulix.
///
/// Windows: %ProgramData%\.rulix
/// Unix-like: /etc/.rulix
pub static SYSTEM_CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let extension = ".rulix";

    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("ProgramData")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\ProgramData"));

        base.join(extension)
    }

    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from("/etc").join(extension)
    }
});
