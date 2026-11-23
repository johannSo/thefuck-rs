use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        ["rebase", "--continue"].iter().all(|required| command.script_parts().iter().any(|part| part == required))
            && command.output().contains("No changes - did you forget to use 'git add'?")
    })
}

fn get_new_command(_command: &Command) -> Vec<String> {
    one("git rebase --skip")
}

pub const RULE: Rule = Rule::new("git_rebase_no_changes", match_, get_new_command);
