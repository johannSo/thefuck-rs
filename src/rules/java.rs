//! Fixes common java command mistake
//!
//! Example:
//! ```text
//! > java foo.java
//! Error: Could not find or load main class foo.java
//! ```

use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["java"]) && command.script.ends_with(".java")
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![command.script[..command.script.len() - 5].to_string()]
}

pub const RULE: Rule = Rule::new("java", match_, get_new_command);
