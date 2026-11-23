use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        let parts = command.script_parts();
        ["rm", "/"].iter().all(|required| parts.iter().any(|part| part == required))
            && !command.script.contains("--no-preserve-root")
            && command.output().contains("--no-preserve-root")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| vec![format!("{} --no-preserve-root", command.script)])
}

pub const RULE: Rule = Rule { enabled_by_default: never, ..Rule::new("rm_root", match_, get_new_command) };
