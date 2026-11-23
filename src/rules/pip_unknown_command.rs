use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        for_app(command, &["pip", "pip2", "pip3"])
            && command.script.contains("pip")
            && command.output().contains("unknown command")
            && command.output().contains("maybe you meant")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    let broken_cmd = &findall(r#"ERROR: unknown command "([^"]+)""#, command.output())[0];
    let new_cmd = &findall(r#"maybe you meant "([^"]+)""#, command.output())[0];

    vec![replace_argument(&command.script, broken_cmd, new_cmd)]
}

pub const RULE: Rule = Rule::new("pip_unknown_command", match_, get_new_command);
