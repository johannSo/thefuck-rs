use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["brew"])
        && command.script_parts().iter().any(|part| part == "install")
        && command.output().contains("brew cask install")
}

fn get_cask_install_lines(output: &str) -> Vec<&str> {
    output.split('\n').map(str::trim).filter(|line| line.starts_with("brew cask install")).collect()
}

fn get_script_for_brew_cask(output: &str) -> String {
    let cask_install_lines = get_cask_install_lines(output);
    if cask_install_lines.len() > 1 {
        and_(&cask_install_lines)
    } else {
        cask_install_lines[0].to_string()
    }
}

fn get_new_command(command: &Command) -> Vec<String> {
    let brew_cask_script = get_script_for_brew_cask(command.output());
    vec![and_(&[&brew_cask_script, &command.script])]
}

pub const RULE: Rule =
    Rule { enabled_by_default: brew_available, ..Rule::new("brew_cask_dependency", match_, get_new_command) };
