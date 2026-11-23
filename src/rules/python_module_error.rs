use super::prelude::*;

const MISSING_MODULE: &str = r"ModuleNotFoundError: No module named '([^']+)'";

fn match_(command: &Command) -> bool {
    command.output().contains("ModuleNotFoundError: No module named '")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let missing_module = &findall(MISSING_MODULE, command.output())[0];
    vec![and_(&[&format!("pip install {}", missing_module), &command.script])]
}

pub const RULE: Rule = Rule::new("python_module_error", match_, get_new_command);
