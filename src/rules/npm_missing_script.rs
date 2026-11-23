use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["npm"])
        && command.script_parts().iter().any(|part| part.starts_with("ru"))
        && command.output().contains("npm ERR! missing script: ")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let misspelled_script = &findall(r".*missing script: (.*)\n", command.output())[0];
    replace_command(command, misspelled_script, get_scripts())
}

pub const RULE: Rule =
    Rule { enabled_by_default: npm_available, ..Rule::new("npm_missing_script", match_, get_new_command) };
