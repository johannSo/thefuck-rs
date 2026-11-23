//! Fixes wrong package names with pacman or yaourt.
//!
//! For example the `llc` program is in package `llvm` so this:
//!     yay -S llc
//! should be:
//!     yay -S llvm

use super::prelude::*;

fn match_(command: &Command) -> bool {
    let parts = command.script_parts();
    !parts.is_empty()
        && (["pacman", "yay", "pikaur", "yaourt"].contains(&parts[0].as_str())
            || parts.get(..2).is_some_and(|start| start == ["sudo", "pacman"]))
        && command.output().contains("error: target not found:")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let parts = command.script_parts();
    let pgr = &parts[parts.len() - 1];

    replace_command(command, pgr, &get_pkgfile(pgr))
}

pub const RULE: Rule =
    Rule { enabled_by_default: archlinux_env, ..Rule::new("pacman_not_found", match_, get_new_command) };
