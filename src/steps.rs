//! Configuration parsing and types for Rulix workflows.
//!
//! This module handles the deserialization of sequential engine tasks (steps)
//! from your YAML rules file. It provides an extensible, untagged pipeline architecture 
//! allowing actions like file matching, moving, and system notifications to be executed 
//! in top-to-bottom sequence.

use serde::Deserialize;

// =========================================================================
// Core Domain Types
// =========================================================================

/// Represents a single sequential action executable by the Rulix engine.
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
    /// The target file extension (e.g., `".pdf"`, `".docx"`).
    pub ext: String,
}

// =========================================================================
// Test Helper Implementations
// =========================================================================

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