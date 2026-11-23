use std::fs;

use super::prelude::*;

fn get_all_environments() -> Vec<String> {
    let root = expanduser("~/.virtualenvs");
    fs::read_dir(root)
        .map(|children| {
            children
                .flatten()
                .filter(|child| child.path().is_dir())
                .map(|child| child.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn match_(command: &Command) -> bool {
    for_app(command, &["workon"])
        && command.script_parts().len() >= 2
        && !get_all_environments().contains(&command.script_parts()[1])
}

fn get_new_command(command: &Command) -> Vec<String> {
    let misspelled_env = &command.script_parts()[1];
    let create_new = format!("mkvirtualenv {}", misspelled_env);

    let available = get_all_environments();
    if available.is_empty() {
        vec![create_new]
    } else {
        let mut new_commands = replace_command(command, misspelled_env, &available);
        new_commands.push(create_new);
        new_commands
    }
}

pub const RULE: Rule = Rule::new("workon_doesnt_exists", match_, get_new_command);
