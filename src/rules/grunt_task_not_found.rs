use super::prelude::*;

const REGEX: &str = r#"Warning: Task "(.*)" not found."#;

fn match_(command: &Command) -> bool {
    for_app(command, &["grunt"]) && regex(REGEX).is_match(command.output())
}

fn parse_tasks(help: &str) -> Vec<String> {
    let mut should_yield = false;
    let mut tasks = Vec::new();
    for line in help.split_inclusive('\n') {
        let line = line.trim();

        if line.contains("Available tasks") {
            should_yield = true;
            continue;
        }

        if should_yield && line.is_empty() {
            break;
        }

        if line.contains("  ") {
            tasks.push(line.split(' ').next().unwrap_or("").to_string());
        }
    }
    tasks
}

fn get_all_tasks() -> Vec<String> {
    cache_strings("grunt_task_not_found.get_all_tasks", &["Gruntfile.js"], || {
        parse_tasks(&run("grunt", &["--help"]).stdout)
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    let misspelled_task = findall(REGEX, command.output())[0].split(':').next().unwrap_or("").to_string();
    let tasks = get_all_tasks();
    let Some(fixed) = get_closest(&misspelled_task, &tasks, 0.6, true) else { return vec![] };
    vec![command.script.replace(&format!(" {}", misspelled_task), &format!(" {}", fixed))]
}

pub const RULE: Rule = Rule::new("grunt_task_not_found", match_, get_new_command);

#[cfg(test)]
mod tests {
    use super::parse_tasks;

    #[test]
    fn parses_tasks() {
        let help = "Grunt: The JavaScript Task Runner (v0.4.5)\n\nUsage\n grunt [options] [task [task ...]]\n\nOptions\n    --help, -h  Display this help text.\n\nAvailable tasks\n  autoprefixer  Prefix CSS files. *\n    concurrent  Run grunt tasks concurrently *\n       compass  Compile Sass to CSS using Compass *\n\nFor more information, see http://gruntjs.com/\n";
        assert_eq!(parse_tasks(help), ["--help,", "autoprefixer", "concurrent", "compass"]);
    }
}
