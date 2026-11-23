use super::prelude::*;

fn match_(command: &Command) -> bool {
    command.script == "test.py" && command.output().contains("not found")
}

fn get_new_command(_command: &Command) -> Vec<String> {
    one("pytest")
}

// make it come before the python_command rule
pub const RULE: Rule = Rule { priority: 900, ..Rule::new("test.py", match_, get_new_command) };
