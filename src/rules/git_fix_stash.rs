use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        let parts = command.script_parts();
        parts.len() > 1 && parts[1] == "stash" && command.output().contains("usage:")
    })
}

const STASH_COMMANDS: &[&str] = &["apply", "branch", "clear", "drop", "list", "pop", "save", "show"];

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        let stash_cmd = &command.script_parts()[2];
        let fixed = get_closest(stash_cmd, STASH_COMMANDS, 0.6, false);

        match fixed {
            Some(fixed) => vec![replace_argument(&command.script, stash_cmd, &fixed)],
            None => {
                let mut cmd = command.script_parts().to_vec();
                cmd.insert(2, "save".to_string());
                vec![cmd.join(" ")]
            }
        }
    })
}

pub const RULE: Rule = Rule::new("git_fix_stash", match_, get_new_command);
