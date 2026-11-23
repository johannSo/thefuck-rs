use super::prelude::*;

const PATTERNS: &[&str] = &[
    r"no such file or directory: (.*)\n?\z",
    r"cannot access '(.*)': No such file or directory",
    r": (.*): No such file or directory",
    r"can't cd to (.*)\n?\z",
];

fn get_destination(command: &Command) -> Option<String> {
    PATTERNS
        .iter()
        .filter_map(|pattern| find_first(pattern, command.output()))
        .find(|found| command.script_parts().contains(found))
}

fn match_(command: &Command) -> bool {
    get_destination(command).is_some_and(|destination| !destination.is_empty())
}

/// Returns absolute paths used in history, the most used first.
fn get_all_absolute_paths_from_history(command: &Command) -> Vec<String> {
    let mut counter: Vec<(String, usize)> = Vec::new();

    for line in get_valid_history_without_current(command) {
        let splitted = shell().split_command(&line);

        for param in splitted.iter().skip(1) {
            if param.starts_with('/') || param.starts_with('~') {
                let param = param.strip_suffix('/').unwrap_or(param);
                match counter.iter_mut().find(|(path, _)| path == param) {
                    Some((_, count)) => *count += 1,
                    None => counter.push((param.to_string(), 1)),
                }
            }
        }
    }

    // The sort is stable: equally popular paths stay in order of appearance.
    counter.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    counter.into_iter().map(|(path, _)| path).collect()
}

fn get_new_command(command: &Command) -> Vec<String> {
    let Some(destination) = get_destination(command) else { return vec![] };
    let paths = get_all_absolute_paths_from_history(command);

    paths
        .iter()
        .filter(|path| path.ends_with(&destination) && expanduser(path).exists())
        .map(|path| replace_argument(&command.script, &destination, path))
        .collect()
}

pub const RULE: Rule = Rule { priority: 800, ..Rule::new("path_from_history", match_, get_new_command) };
