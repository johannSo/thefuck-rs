//! Subcommands of applications, so a mistyped subcommand can be fixed right
//! away together with a mistyped application, like `gti atus` -> `git status`.
//!
//! The lists come from the applications themselves, but only from listing
//! commands that don't do anything else: the corrected command is never run
//! to find out if it fails.

use crate::types::Command;
use crate::utils::{get_close_matches, replace_argument, run, which};

/// Returns subcommands of the application, empty when they aren't known.
pub fn known_subcommands(app: &str) -> Vec<String> {
    let basename = app.rsplit('/').next().unwrap_or(app);
    if which(app).is_none() {
        return vec![];
    }
    match basename {
        // Repository and global aliases included, they're valid subcommands too.
        "git" | "hub" => run(app, &["--list-cmds=main,others,alias,nohelpers"])
            .stdout
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(String::from)
            .collect(),
        "cargo" => parse_cargo_list(&run(app, &["--list"]).stdout),
        _ => vec![],
    }
}

/// Parses `cargo --list`: the first word of every indented line.
fn parse_cargo_list(output: &str) -> Vec<String> {
    output
        .lines()
        .filter(|line| line.starts_with(char::is_whitespace))
        .filter_map(|line| line.split_whitespace().next())
        .map(String::from)
        .collect()
}

/// Returns scripts of the command with its subcommand fixed, using the
/// subcommands of its application. The script is returned as is when its
/// subcommand is fine, unknown or can't be fixed.
pub fn fix_subcommand(command: &Command) -> Vec<String> {
    fix_subcommand_using(&command.script, command.script_parts(), known_subcommands)
}

fn fix_subcommand_using(
    script: &str,
    script_parts: &[String],
    subcommands_of: impl FnOnce(&str) -> Vec<String>,
) -> Vec<String> {
    let unchanged = || vec![script.to_string()];
    let [app, subcommand, ..] = script_parts else {
        return unchanged();
    };
    // Options can go before the subcommand and take values, don't guess there.
    if subcommand.starts_with('-') {
        return unchanged();
    }
    let subcommands = subcommands_of(app);
    if subcommands.is_empty() || subcommands.contains(subcommand) {
        return unchanged();
    }
    let fixes: Vec<String> = get_close_matches(subcommand, &subcommands, 0.6)
        .iter()
        .map(|fixed| replace_argument(script, subcommand, fixed))
        .collect();
    if fixes.is_empty() {
        unchanged()
    } else {
        fixes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix(script: &str) -> Vec<String> {
        let parts: Vec<String> = script.split(' ').map(String::from).collect();
        fix_subcommand_using(script, &parts, |app| {
            assert_eq!(app, "git");
            ["status", "stash", "commit", "st"]
                .map(String::from)
                .to_vec()
        })
    }

    #[test]
    fn fixes_mistyped_subcommand() {
        assert_eq!(fix("git atus"), ["git status"]);
        assert_eq!(fix("git comit -m atus"), ["git commit -m atus"]);
    }

    #[test]
    fn keeps_valid_or_unfixable_subcommands() {
        assert_eq!(fix("git status"), ["git status"]);
        assert_eq!(fix("git st"), ["git st"]);
        assert_eq!(fix("git xyzzyq"), ["git xyzzyq"]);
        assert_eq!(fix("git -C atus"), ["git -C atus"]);
        assert_eq!(fix("git"), ["git"]);
    }

    #[test]
    fn keeps_commands_of_unknown_apps() {
        let parts = ["ls".to_string(), "atus".to_string()];
        assert_eq!(
            fix_subcommand_using("ls atus", &parts, |_| vec![]),
            ["ls atus"]
        );
    }

    #[test]
    fn parses_cargo_list() {
        let output = "Installed Commands:\n    add                  Add dependencies\n    b                    alias: build\n    build                Compile\n";
        assert_eq!(parse_cargo_list(output), ["add", "b", "build"]);
    }
}
