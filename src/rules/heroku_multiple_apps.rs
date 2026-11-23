use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["heroku"])
        && command.output().contains("https://devcenter.heroku.com/articles/multiple-environments")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let apps = findall(r"([^ ]*) \([^)]*\)", command.output());
    apps.iter().map(|app| format!("{} --app {}", command.script, app)).collect()
}

pub const RULE: Rule = Rule::new("heroku_multiple_apps", match_, get_new_command);
