use super::prelude::*;

const SUPPORTED_APPS: &[&str] = &["goenv", "nodenv", "pyenv", "rbenv"];

fn enabled_by_default() -> bool {
    SUPPORTED_APPS.iter().any(|app| which(app).is_some())
}

fn common_typos(broken: &str) -> &'static [&'static str] {
    match broken {
        "list" => &["versions", "install --list"],
        "remove" => &["uninstall"],
        _ => &[],
    }
}

fn match_(command: &Command) -> bool {
    is_app(command, SUPPORTED_APPS, 1) && command.output().contains("env: no such command ")
}

fn get_app_commands(app: &str) -> Vec<String> {
    cache_for_app(app, &format!("omnienv_no_such_command.get_app_commands:{}", app), || {
        run(app, &["commands"]).stdout.lines().map(|line| line.trim().to_string()).collect()
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    let broken = &findall(r"env: no such command ['`]([^']*)'", command.output())[0];
    let mut matched: Vec<String> = common_typos(broken)
        .iter()
        .map(|common_typo| replace_argument(&command.script, broken, common_typo))
        .collect();

    let app = &command.script_parts()[0];
    let app_commands = get_app_commands(app);
    matched.extend(replace_command(command, broken, &app_commands));
    matched
}

pub const RULE: Rule = Rule { enabled_by_default, ..Rule::new("omnienv_no_such_command", match_, get_new_command) };
