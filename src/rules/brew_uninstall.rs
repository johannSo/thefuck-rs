use super::prelude::*;

fn match_(command: &Command) -> bool {
    is_app(command, &["brew"], 2)
        && ["uninstall", "rm", "remove"].contains(&command.script_parts()[1].as_str())
        && command.output().contains("brew uninstall --force")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let mut command_parts = command.script_parts().to_vec();
    command_parts[1] = "uninstall".to_string();
    command_parts.insert(2, "--force".to_string());
    vec![command_parts.join(" ")]
}

pub const RULE: Rule = Rule::new("brew_uninstall", match_, get_new_command);
