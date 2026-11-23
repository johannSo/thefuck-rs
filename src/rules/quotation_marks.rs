//! Fixes careless " and ' usage
//!
//! Example:
//! ```text
//! > git commit -m 'My Message"
//! ```

use super::prelude::*;

fn match_(command: &Command) -> bool {
    command.script.contains('\'') && command.script.contains('"')
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![command.script.replace('\'', "\"")]
}

pub const RULE: Rule = Rule::new("quotation_marks", match_, get_new_command);
