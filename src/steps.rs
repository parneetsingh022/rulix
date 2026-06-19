//! Workflow configuration types.
//!
//! Defines the structures used to deserialize the `steps` section of a Rulix
//! rule file. Each YAML step maps to a [`Step`] variant, which represents a
//! single action in the rule execution pipeline.
//!
//! Steps are executed sequentially in the order they appear in the
//! configuration file. Each step represents a discrete operation within
//! the workflow pipeline.
//!
//! This module serves as the configuration layer between YAML rule files and
//! the runtime execution engine.

use serde::Deserialize;

/// Defines a single operation within a rule's workflow pipeline.
///
/// Each step is deserialized from YAML and executed sequentially by
/// the engine as part of rule processing.
#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum Step {
    /// Filters the engine stream based on file attributes.
    Match {
        #[serde(rename = "match")]
        criteria: MatchCriteria,
    },

    /// Relocates the targeted file to a specified destination directory.
    MoveTo { 
        move_to: String 
    },

    /// Dispatches a desktop or terminal alert message to the user.
    Notify { 
        notify: String 
    },
}

/// Rule filters used to evaluate whether a file matches a given `Step::Match`.
#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct MatchCriteria {
    /// The target file extension (e.g., `"pdf"`, `"docx"`).
    pub ext: String,
}

/// Factory constructors for cleanly instantiating specific `Step` variants.
///
/// These helpers provide a higher-level API to build individual steps without having to 
/// manually construct the raw, underlying struct or enum variants throughout the codebase.
///
/// # Scope & Future Evolution
/// * **Current Status:** Restricted to `#[cfg(test)]` because these are currently only needed 
///   within the test suite to simplify test setup.
/// * **Future Roadmap:** This implementation block may be exposed outside of tests (`#[cfg(test)]` removed) 
/// in a future iteration when rule building via the CLI interface is implemented.
#[cfg(test)]
impl Step {
    /// Helper factory to cleanly construct a `Step::Match` variant in unit tests.
    pub fn new_match(ext: &str) -> Self {
        Self::Match {
            criteria: MatchCriteria {
                ext: ext.to_string(),
            },
        }
    }

    /// Helper factory to cleanly construct a `Step::MoveTo` variant in unit tests.
    pub fn new_move_to(path: &str) -> Self {
        Self::MoveTo {
            move_to: path.to_string(),
        }
    }

    /// Helper factory to cleanly construct a `Step::Notify` variant in unit tests.
    pub fn new_notify(msg: &str) -> Self {
        Self::Notify {
            notify: msg.to_string(),
        }
    }
}