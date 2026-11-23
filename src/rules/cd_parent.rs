//! Adds the missing space between the cd command and the target directory
//! when trying to cd to the parent directory.
//!
//! Does not really save chars, but is fun :D
//!
//! Example:
//! ```text
//! > cd..
//! cd..: command not found
//! ```

use super::prelude::*;

fn match_(command: &Command) -> bool {
    command.script == "cd.."
}

fn get_new_command(_command: &Command) -> Vec<String> {
    one("cd ..")
}

pub const RULE: Rule = Rule::new("cd_parent", match_, get_new_command);
