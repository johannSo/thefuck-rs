//! Redirects cs to cd when there is a typo.
//! Due to the proximity of the keys - d and s - this seems like a common typo
//!
//! ```text
//! ~ > cs /etc/
//! cs: command not found
//! ~ > fuck
//! cd /etc/ [enter/↑/↓/ctrl+c]
//! /etc >
//! ```

use super::prelude::*;

fn match_(command: &Command) -> bool {
    command.script_parts()[0] == "cs"
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![format!("cd{}", command.script.chars().skip(2).collect::<String>())]
}

pub const RULE: Rule = Rule { priority: 900, ..Rule::new("cd_cs", match_, get_new_command) };
