use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script.contains(" rm ")
            && command.output().contains("fatal: not removing '")
            && command.output().contains("' recursively without -r")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        let mut command_parts = command.script_parts().to_vec();
        let index = command_parts.iter().position(|part| part == "rm").expect("rm is in the script") + 1;
        command_parts.insert(index, "-r".to_string());
        vec![command_parts.join(" ")]
    })
}

pub const RULE: Rule = Rule::new("git_rm_recursive", match_, get_new_command);
