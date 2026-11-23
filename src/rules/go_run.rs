//! Appends .go when compiling go files
//!
//! Example:
//! ```text
//! > go run foo
//! error: go run: no go files listed
//! ```

use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["go"]) && command.script.starts_with("go run ") && !command.script.ends_with(".go")
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![format!("{}.go", command.script)]
}

pub const RULE: Rule = Rule::new("go_run", match_, get_new_command);
