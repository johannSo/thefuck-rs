use super::prelude::*;

const REGEX: &str = r"Task '(.*)' (is ambiguous|not found)";

fn match_(command: &Command) -> bool {
    for_app(command, &["gradle", "gradlew"]) && regex(REGEX).is_match(command.output())
}

fn parse_tasks(output: &str) -> Vec<String> {
    let mut should_yield = false;
    let mut tasks = Vec::new();
    for line in output.split_inclusive('\n') {
        let line = line.trim();
        if line.starts_with("----") {
            should_yield = true;
            continue;
        }

        if line.is_empty() {
            should_yield = false;
            continue;
        }

        if should_yield && !line.starts_with("All tasks runnable from root project") {
            tasks.push(line.split(' ').next().unwrap_or("").to_string());
        }
    }
    tasks
}

fn get_all_tasks(gradle: &str) -> Vec<String> {
    parse_tasks(&run(gradle, &["tasks"]).stdout)
}

fn get_new_command(command: &Command) -> Vec<String> {
    let wrong_task = &findall(REGEX, command.output())[0];
    let all_tasks = get_all_tasks(&command.script_parts()[0]);
    replace_command(command, wrong_task, &all_tasks)
}

pub const RULE: Rule = Rule::new("gradle_no_task", match_, get_new_command);

#[cfg(test)]
mod tests {
    use super::parse_tasks;

    #[test]
    fn parses_tasks() {
        let output = ":tasks\n\n------------------------------------------------------------\nAll tasks runnable from root project\n------------------------------------------------------------\n\nAndroid tasks\n-------------\nandroidDependencies - Displays the Android dependencies of the project.\nsigningReport - Displays the signing info for each variant.\n\nBuild tasks\n-----------\nassemble - Assembles all variants of all applications and secondary packages.\n\nTo see all tasks and more detail, run gradlew tasks --all\n";
        assert_eq!(parse_tasks(output), ["androidDependencies", "signingReport", "assemble"]);
    }
}
