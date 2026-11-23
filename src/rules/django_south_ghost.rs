use super::prelude::*;

fn match_(command: &Command) -> bool {
    command.script.contains("manage.py")
        && command.script.contains("migrate")
        && command.output().contains("or pass --delete-ghost-migrations")
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![format!("{} --delete-ghost-migrations", command.script)]
}

pub const RULE: Rule = Rule::new("django_south_ghost", match_, get_new_command);
