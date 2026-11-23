use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["vagrant"]) && command.output().to_lowercase().contains("run `vagrant up`")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let cmds = command.script_parts();
    let machine = cmds.get(2);

    let start_all_instances = and_(&["vagrant up", &command.script]);
    match machine {
        None => vec![start_all_instances],
        Some(machine) => vec![and_(&[&format!("vagrant up {}", machine), &command.script]), start_all_instances],
    }
}

pub const RULE: Rule = Rule::new("vagrant_up", match_, get_new_command);
