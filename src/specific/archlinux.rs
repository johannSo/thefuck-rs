//! This file provide some utility functions for Arch Linux specific rules.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::utils::{run, which};

/// Gets the packages that provide the given command using `pkgfile`.
///
/// If the command is of the form `sudo foo`, searches for the `foo` command
/// instead.
pub fn get_pkgfile(command: &str) -> Vec<String> {
    static CACHE: Mutex<Option<HashMap<String, Vec<String>>>> = Mutex::new(None);

    let command = command.trim();
    let command = command.strip_prefix("sudo ").unwrap_or(command);
    let command = command.split(' ').next().unwrap_or("");

    let mut cache = CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    cache
        .get_or_insert_with(HashMap::new)
        .entry(command.to_string())
        .or_insert_with(|| {
            let output = run("pkgfile", &["-b", "-v", command]);
            if !output.success && !(output.code == Some(1) && output.stdout.is_empty()) {
                panic!("pkgfile failed: {}", output.stderr.trim());
            }
            output
                .stdout
                .lines()
                .filter_map(|package| package.split_whitespace().next())
                .map(String::from)
                .collect()
        })
        .clone()
}

/// Returns the package manager to use, if any.
pub fn pacman() -> Option<&'static str> {
    if which("yay").is_some() {
        Some("yay")
    } else if which("pikaur").is_some() {
        Some("pikaur")
    } else if which("yaourt").is_some() {
        Some("yaourt")
    } else if which("pacman").is_some() {
        Some("sudo pacman")
    } else {
        None
    }
}

/// Returns `true` when rules needing `pkgfile` can be enabled.
pub fn archlinux_env() -> bool {
    pacman().is_some() && which("pkgfile").is_some()
}
