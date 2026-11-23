//! Suggest creating symbolic link if hard link is not allowed.
//!
//! Example:
//! ```text
//! > ln barDir barLink
//! ln: ‘barDir’: hard link not allowed for directory
//! ```
//!
//! --> ln -s barDir barLink

use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        command.output().ends_with("hard link not allowed for directory") && command.script_parts()[0] == "ln"
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| vec![regex("^ln ").replace(&command.script, "ln -s ").into_owned()])
}

pub const RULE: Rule = Rule::new("ln_no_hard_link", match_, get_new_command);
