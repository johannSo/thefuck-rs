use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script_parts().iter().any(|part| part == "bisect")
            && command.output().contains("usage: git bisect")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        let broken = &findall(r"git bisect ([^ $]*).*", &command.script)[0];
        let usage = &findall(r"usage: git bisect \[([^\]]+)\]", command.output())[0];
        replace_command(command, broken, &usage.split('|').collect::<Vec<_>>())
    })
}

pub const RULE: Rule = Rule::new("git_bisect_usage", match_, get_new_command);
