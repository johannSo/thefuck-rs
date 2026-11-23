use super::prelude::*;

const REGEX: &str = r"nix-env -iA ([^\s]*)";

fn match_(command: &Command) -> bool {
    regex(REGEX).is_match(command.output())
}

fn get_new_command(command: &Command) -> Vec<String> {
    let name = &findall(REGEX, command.output())[0];
    vec![and_(&[&format!("nix-env -iA {}", name), &command.script])]
}

pub const RULE: Rule =
    Rule { enabled_by_default: nix_available, ..Rule::new("nixos_cmd_not_found", match_, get_new_command) };
