use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        for_app(command, &["lein"])
            && command.script.starts_with("lein")
            && command.output().contains("is not a task. See 'lein help'")
            && command.output().contains("Did you mean this?")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| {
        let broken_cmd = &findall(r"'([^']*)' is not a task", command.output())[0];
        let new_cmds = get_all_matched_commands(command.output(), &["Did you mean this?"]);
        replace_command(command, broken_cmd, &new_cmds)
    })
}

pub const RULE: Rule = Rule::new("lein_not_task", match_, get_new_command);
