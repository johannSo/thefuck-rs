use std::path::Path;

use super::prelude::*;

fn match_(command: &Command) -> bool {
    is_app(command, &["cat"], 1)
        && command.output().starts_with("cat: ")
        && Path::new(&command.script_parts()[1]).is_dir()
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![command.script.replacen("cat", "ls", 1)]
}

pub const RULE: Rule = Rule::new("cat_dir", match_, get_new_command);
