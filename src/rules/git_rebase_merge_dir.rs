use super::prelude::*;
use crate::difflib;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script.contains(" rebase")
            && command.output().contains("It seems that there is already a rebase-merge directory")
            && command.output().contains("I wonder if you are in the middle of another rebase")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        let mut command_list = strings(&["git rebase --continue", "git rebase --abort", "git rebase --skip"]);
        let lines: Vec<&str> = command.output().split('\n').collect();
        let rm_cmd = lines[lines.len() - 4];
        command_list.push(rm_cmd.trim().to_string());
        difflib::get_close_matches(&command.script, &command_list, 4, 0.0)
    })
}

pub const RULE: Rule = Rule::new("git_rebase_merge_dir", match_, get_new_command);
