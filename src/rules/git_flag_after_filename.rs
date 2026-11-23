use super::prelude::*;

const ERROR_PATTERN: &str = "fatal: bad flag '(.*?)' used after filename";
const ERROR_PATTERN2: &str = "fatal: option '(.*?)' must come before non-option arguments";

fn bad_flag(command: &Command) -> Option<String> {
    find_first(ERROR_PATTERN, command.output()).or_else(|| find_first(ERROR_PATTERN2, command.output()))
}

fn match_(command: &Command) -> bool {
    git_support(command, |command| bad_flag(command).is_some())
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        let mut command_parts = command.script_parts().to_vec();

        // find the filename that goes before the bad flag
        let Some(bad_flag) = bad_flag(command) else { return vec![] };
        let bad_flag_index = command_parts.iter().position(|part| *part == bad_flag).expect("flag is in the script");
        let filename_index = (0..bad_flag_index)
            .rev()
            .find(|index| !command_parts[*index].starts_with('-'))
            .expect("file name goes before the flag");

        // swap them
        command_parts.swap(bad_flag_index, filename_index);

        vec![command_parts.join(" ")]
    })
}

pub const RULE: Rule = Rule::new("git_flag_after_filename", match_, get_new_command);
