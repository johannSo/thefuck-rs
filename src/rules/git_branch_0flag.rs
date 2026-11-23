use super::prelude::*;

fn first_0flag(script_parts: &[String]) -> Option<&String> {
    script_parts.iter().find(|p| p.chars().count() == 2 && p.starts_with('0'))
}

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script_parts()[1] == "branch" && first_0flag(command.script_parts()).is_some()
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        let Some(branch_name) = first_0flag(command.script_parts()) else { return vec![] };
        let fixed_flag = branch_name.replace('0', "-");
        let fixed_script = command.script.replace(branch_name.as_str(), &fixed_flag);
        if command.output().contains("A branch named '") && command.output().contains("' already exists.") {
            let delete_branch = format!("git branch -D {}", branch_name);
            return vec![and_(&[&delete_branch, &fixed_script])];
        }
        vec![fixed_script]
    })
}

pub const RULE: Rule = Rule::new("git_branch_0flag", match_, get_new_command);
