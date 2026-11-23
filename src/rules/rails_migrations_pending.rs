use super::prelude::*;

const SUGGESTION_REGEX: &str = r"To resolve this issue, run:\s+(.*?)\n";

fn match_(command: &Command) -> bool {
    command.output().contains("Migrations are pending. To resolve this issue, run:")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let migration_script = &findall(SUGGESTION_REGEX, command.output())[0];
    vec![and_(&[migration_script, &command.script])]
}

pub const RULE: Rule = Rule::new("rails_migrations_pending", match_, get_new_command);
