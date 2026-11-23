use super::prelude::*;

fn suggests_require(command: &Command) -> bool {
    command.script_parts().iter().any(|part| part == "install")
        && command.output().to_lowercase().contains("composer require")
}

fn match_(command: &Command) -> bool {
    let output = command.output().to_lowercase();
    for_app(command, &["composer"])
        && (output.contains("did you mean this?")
            || output.contains("did you mean one of these?")
            || suggests_require(command))
}

fn get_new_command(command: &Command) -> Vec<String> {
    let (broken_cmd, new_cmd) = if suggests_require(command) {
        ("install".to_string(), "require".to_string())
    } else {
        let broken_cmd = findall(r#"Command "([^']*)" is not defined"#, command.output())[0].clone();
        let mut new_cmd = findall(r"Did you mean this\?[^\n]*\n\s*([^\n]*)", command.output());
        if new_cmd.is_empty() {
            new_cmd = findall(r"Did you mean one of these\?[^\n]*\n\s*([^\n]*)", command.output());
        }
        (broken_cmd, new_cmd[0].trim().to_string())
    };
    vec![replace_argument(&command.script, &broken_cmd, &new_cmd)]
}

pub const RULE: Rule = Rule::new("composer_not_command", match_, get_new_command);
