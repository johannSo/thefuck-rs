use super::prelude::*;

fn match_(command: &Command) -> bool {
    command.script.contains("manage.py")
        && command.script.contains("migrate")
        && command.output().contains("--merge: will just attempt the migration")
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![format!("{} --merge", command.script)]
}

pub const RULE: Rule = Rule::new("django_south_merge", match_, get_new_command);
