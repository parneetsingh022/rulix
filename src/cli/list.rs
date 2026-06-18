//! This module implements the logic behind the `rulix --list` command.
//!
//! The command reads all rule definitions from the user's `rules.yaml`
//! configuration file and displays them in a human-readable format.
//!
//! Each configured rule is loaded and rendered in a structured layout,
//! allowing users to quickly inspect the rules currently recognized by
//! Rulix. The presentation is intended to be easy to scan and understand,
//! providing a convenient overview of the active configuration.
//!
//! This command serves as a simple way to audit the current rule set and
//! verify that the configuration has been loaded as expected, without
//! requiring users to manually inspect the configuration file.

use anyhow::Result;

use std::borrow::Cow;

use crate::rules::{RulesSource, RulixRules};
use crate::errors::FileError;

/// Displays all rules defined in the provided rules file.
///
/// The rules are loaded from `rules`, after which a summary of the
/// loaded rule set is printed to standard output.
///
/// # Errors
///
/// Returns an error if the configuration file cannot be read, parsed, or
/// validated.
pub fn run(source: RulesSource) -> Result<()> {
    let rules_path = source.path();

    let rules = match RulixRules::from_file(rules_path) {
        Ok(rules) => rules,

        // When `--rules` is not provided, Rulix falls back to its default rules file.
        // That file may not exist yet, especially on first startup, so a missing
        // default rules file is not treated as an error.
        //
        // If the user explicitly provides a path with `--rules`, then that file is
        // expected to exist and a missing file should be reported as an error.
        Err(FileError::NotFound(_)) if !source.is_user_provided() => {
            println!("No rules to show.");
            return Ok(())
        },

        Err(err) => return Err(err.into())
    };

    let space = "    ";
    let max_name_length = 30;

    println!("Rulix Configuration");

    println!("{space}File: {}", rules_path.display());
    println!("{space}Rules: {}", rules.len());
    println!();

    println!("Available Rules");

    for (i, rule) in rules.rules.iter().enumerate() {
        println!(
            "{space}[{i}] {name:<name_width$}",
            name = truncate_with_ellipsis(&rule.name, max_name_length),
            name_width = max_name_length
        );
    }

    Ok(())
}

/// Truncates a UTF-8 string to at most `max_bytes` bytes and appends `...`
/// when truncation occurs.
///
/// The returned string is guaranteed to end on a valid UTF-8 character boundary.
/// If `s` already fits within `max_bytes`, this returns a borrowed `&str`
/// without allocating. If truncation is needed, it returns an owned `String`.
///
/// The `max_bytes` value includes the ellipsis. For example, with
/// `max_bytes = 10`, the result may contain up to 7 bytes from `s` plus `...`.
fn truncate_with_ellipsis(s: &str, max_bytes: usize) -> Cow<'_, str> {
    if s.len() <= max_bytes {
        return Cow::Borrowed(s);
    }

    if max_bytes <= 3 {
        return Cow::Owned(".".repeat(max_bytes));
    }

    let cutoff = s.floor_char_boundary(max_bytes - 3);

    let mut out = String::with_capacity(cutoff + 3);
    out.push_str(&s[..cutoff]);
    out.push_str("...");
    Cow::Owned(out)
}
