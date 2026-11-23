use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.output().contains("fatal: A branch named '") && command.output().contains("' already exists.")
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        let branch_name = findall(r"fatal: A branch named '(.+)' already exists.", command.output())[0]
            .replace('\'', r"\'");
        let new_command_templates: [&[&str]; 5] = [
            &["git branch -d {0}", "git branch {0}"],
            &["git branch -d {0}", "git checkout -b {0}"],
            &["git branch -D {0}", "git branch {0}"],
            &["git branch -D {0}", "git checkout -b {0}"],
            &["git checkout {0}"],
        ];
        new_command_templates
            .iter()
            .map(|new_command_template| and_(new_command_template).replace("{0}", &branch_name))
            .collect()
    })
}

pub const RULE: Rule = Rule::new("git_branch_exists", match_, get_new_command);
