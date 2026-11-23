use super::prelude::*;

fn match_(command: &Command) -> bool {
    let output = command.output();
    for_app(command, &["cp", "mv"])
        && (output.contains("No such file or directory")
            || (output.starts_with("cp: directory") && output.trim_end().ends_with("does not exist")))
}

fn get_new_command(command: &Command) -> Vec<String> {
    let destination = &command.script_parts()[command.script_parts().len() - 1];
    vec![and_(&[&format!("mkdir -p {}", destination), &command.script])]
}

pub const RULE: Rule = Rule::new("cp_create_destination", match_, get_new_command);
