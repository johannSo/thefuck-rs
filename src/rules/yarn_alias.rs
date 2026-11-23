use super::prelude::*;

fn match_(command: &Command) -> bool {
    is_app(command, &["yarn"], 1) && command.output().contains("Did you mean")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let broken = &command.script_parts()[1];
    let fix = &findall(r#"Did you mean [`"](?:yarn )?([^`"]*)[`"]"#, command.output())[0];

    vec![replace_argument(&command.script, broken, fix)]
}

pub const RULE: Rule = Rule::new("yarn_alias", match_, get_new_command);
