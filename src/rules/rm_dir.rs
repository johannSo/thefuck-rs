use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        command.script.contains("rm") && command.output().to_lowercase().contains("is a directory")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| {
        let arguments = if command.script.contains("hdfs") { "-r" } else { "-rf" };
        let replacement = format!("rm {} ${{1}}", arguments);
        vec![regex(r"\brm (.*)").replace_all(&command.script, replacement.as_str()).into_owned()]
    })
}

pub const RULE: Rule = Rule::new("rm_dir", match_, get_new_command);
