use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Move {
        // Use full, complete absolute paths for clear tracking
        from: PathBuf,
        to: PathBuf,
        is_dir: bool,
        checksum: Option<String>,
    },
}

impl Operation {
    /// Generates a perfectly inverted operation without any risk of panics.
    pub fn undo(&self) -> Self {
        match self {
            Operation::Move {
                from,
                to,
                is_dir,
                checksum,
            } => {
                Operation::Move {
                    // Swapping 'from' and 'to' cleanly reverses the action
                    from: to.clone(),
                    to: from.clone(),
                    is_dir: *is_dir,
                    checksum: checksum.clone(),
                }
            }
        }
    }
}
