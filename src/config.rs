use std::{env, path::PathBuf, sync::LazyLock};

pub static SYSTEM_CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let extension = ".rulix";

    #[cfg(target_os = "windows")]
    {
        let base = env::var("ProgramData")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\ProgramData"));

        base.join(extension)
    }

    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from("/etc").join(extension)
    }
});
