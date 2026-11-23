use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        for_app(command, &["pip"])
            && command.script.contains("pip install")
            && command.output().contains("Permission denied")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    if !command.script.contains("--user") {
        // add --user (attempt 1)
        return vec![command.script.replace(" install ", " install --user ")];
    }

    // since --user didn't fix things, let's try sudo (attempt 2)
    vec![format!("sudo {}", command.script.replace(" --user", ""))]
}

pub const RULE: Rule = Rule::new("pip_install", match_, get_new_command);
