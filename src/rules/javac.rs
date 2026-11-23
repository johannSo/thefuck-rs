//! Appends .java when compiling java files
//!
//! Example:
//! ```text
//! > javac foo
//! error: Class names, 'foo', are only accepted if annotation
//! processing is explicitly requested
//! ```

use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["javac"]) && !command.script.ends_with(".java")
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![format!("{}.java", command.script)]
}

pub const RULE: Rule = Rule::new("javac", match_, get_new_command);
