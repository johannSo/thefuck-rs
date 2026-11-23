//! Appends .py when executing python files
//!
//! Example:
//! ```text
//! > python foo
//! error: python: can't open file 'foo': [Errno 2] No such file or directory
//! ```

use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["python"]) && !command.script.ends_with(".py")
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![format!("{}.py", command.script)]
}

pub const RULE: Rule = Rule::new("python_execute", match_, get_new_command);
