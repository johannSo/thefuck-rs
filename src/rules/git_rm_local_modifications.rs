use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script.contains(" rm ")
            && command.output().contains("error: the following file has local modifications")
            && command.output().contains("use --cached to keep the file, or -f to force removal")
    })
}

pub(super) fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        let mut command_parts = command.script_parts().to_vec();
        let index = command_parts.iter().position(|part| part == "rm").expect("rm is in the script") + 1;
        command_parts.insert(index, "--cached".to_string());
        let mut command_list = vec![command_parts.join(" ")];
        command_parts[index] = "-f".to_string();
        command_list.push(command_parts.join(" "));
        command_list
    })
}

pub const RULE: Rule = Rule::new("git_rm_local_modifications", match_, get_new_command);
