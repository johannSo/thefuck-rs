use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["gulp"]) && command.output().contains("is not in your gulpfile")
}

fn get_gulp_tasks() -> Vec<String> {
    cache_strings("gulp_not_task.get_gulp_tasks", &["gulpfile.js"], || {
        run("gulp", &["--tasks-simple"]).stdout.lines().map(String::from).collect()
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    let wrong_task = &findall(r"Task '(\w+)' is not in your gulpfile", command.output())[0];
    replace_command(command, wrong_task, &get_gulp_tasks())
}

pub const RULE: Rule = Rule::new("gulp_not_task", match_, get_new_command);
