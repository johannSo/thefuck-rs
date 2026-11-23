use super::prelude::*;

const MISTAKE: &str = r#"Terraform has no command named "([^"]+)"\."#;
const FIX: &str = r#"Did you mean "([^"]+)"\?"#;

fn match_(command: &Command) -> bool {
    for_app(command, &["terraform"])
        && regex(MISTAKE).is_match(command.output())
        && regex(FIX).is_match(command.output())
}

fn get_new_command(command: &Command) -> Vec<String> {
    let mistake = &findall(MISTAKE, command.output())[0];
    let fix = &findall(FIX, command.output())[0];
    vec![command.script.replace(mistake.as_str(), fix)]
}

pub const RULE: Rule = Rule::new("terraform_no_command", match_, get_new_command);
