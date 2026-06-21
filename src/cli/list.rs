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

use crate::rules::RuleSet;

/// Displays all rules defined in the provided rules file.
///
/// The rules are loaded from `rules`, after which a summary of the
/// loaded rule set is printed to standard output.
///
/// # Errors
///
/// Returns an error if the configuration file cannot be read, parsed, or
/// validated.
pub fn run(rules: RuleSet) -> Result<()> {
    let indent_size = 4;
    let max_name_length = 30;

    let index_width = rules.rules.len().to_string().len().max(2);

    println!("Rulix Configuration");

    println!(
        "{space:<indent_width$}File: {path}",
        space = "",
        indent_width = indent_size,
        path = rules.path().display()
    );
    println!(
        "{space:<indent_width$}Rules: {count}",
        space = "",
        indent_width = indent_size,
        count = rules.len()
    );

    println!();

    println!("Available Rules\n");

    println!(
        "{space:<indent_width$}{id_hdr:>i_width$}   {name_hdr:<name_width$}   TARGET DIR",
        space = "",
        indent_width = indent_size,
        id_hdr = "ID",
        i_width = index_width,
        name_hdr = "RULE NAME",
        name_width = max_name_length,
    );

    println!();

    // Print Rows
    for (i, rule) in rules.rules.iter().enumerate() {
        println!(
            "{space:<indent_width$}{id:>i_width$}   {name:<name_width$}   {target_hdr}",
            space = "",
            indent_width = indent_size,
            id = i,
            i_width = index_width,
            name = truncate_with_ellipsis(&rule.name, max_name_length),
            name_width = max_name_length,
            target_hdr = rule.target.display()
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
    if max_bytes == 0 {
        return Cow::Borrowed(""); // No allocation for 0 bytes!
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn test_no_truncation_needed() {
        // Fits perfectly, should be Borrowed
        let s = "hello";
        let res = truncate_with_ellipsis(s, 5);
        assert_eq!(res, "hello");
        assert!(matches!(res, Cow::Borrowed(_)));

        // Well under the limit, should be Borrowed
        let res_short = truncate_with_ellipsis(s, 10);
        assert_eq!(res_short, "hello");
        assert!(matches!(res_short, Cow::Borrowed(_)));
    }

    #[test]
    fn test_basic_truncation() {
        // Needs truncation, fits exactly after accounting for "..."
        let s = "hello world"; // 11 bytes
        let res = truncate_with_ellipsis(s, 8); // 5 bytes ("hello") + 3 bytes ("...")
        assert_eq!(res, "hello...");
        assert!(matches!(res, Cow::Owned(_)));
    }

    #[test]
    fn test_utf8_boundary_truncation() {
        // "🦀" is 4 bytes. "🦀🦀" is 8 bytes.
        let crabs = "🦀🦀";

        // Limit 7: Cannot fit second crab. Floors to 1st crab (4 bytes) + "..." (3 bytes) = 7 bytes.
        let res = truncate_with_ellipsis(crabs, 7);
        assert_eq!(res, "🦀...");
        assert!(matches!(res, Cow::Owned(_)));

        // Limit 6: Cannot fit the first crab + "...". Floors to 0 bytes + "..." = 3 bytes.
        let res_tight = truncate_with_ellipsis(crabs, 6);
        assert_eq!(res_tight, "...");
        assert!(matches!(res_tight, Cow::Owned(_)));
    }

    #[test]
    fn test_edge_case_small_max_bytes() {
        let s = "heavy rotation";

        // max_bytes = 3: exactly fits "..."
        let res_3 = truncate_with_ellipsis(s, 3);
        assert_eq!(res_3, "...");
        assert!(matches!(res_3, Cow::Owned(_)));

        // max_bytes = 2: returns ".."
        let res_2 = truncate_with_ellipsis(s, 2);
        assert_eq!(res_2, "..");
        assert!(matches!(res_2, Cow::Owned(_)));

        // max_bytes = 0: returns empty string slice, zero allocations!
        let res_0 = truncate_with_ellipsis(s, 0);
        assert_eq!(res_0, "");
        assert!(matches!(res_0, Cow::Borrowed(_)));
    }

    #[test]
    fn test_empty_string() {
        let s = "";

        // Empty string always fits, should be Borrowed
        let res = truncate_with_ellipsis(s, 5);
        assert_eq!(res, "");
        assert!(matches!(res, Cow::Borrowed(_)));

        let res_zero = truncate_with_ellipsis(s, 0);
        assert_eq!(res_zero, "");
        assert!(matches!(res_zero, Cow::Borrowed(_)));
    }
}
