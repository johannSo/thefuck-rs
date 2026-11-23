use super::prelude::*;

const PATTERNS: &[&str] = &[
    r"mv: cannot move '[^']*' to '([^']*)': No such file or directory",
    r"mv: cannot move '[^']*' to '([^']*)': Not a directory",
    r"cp: cannot create regular file '([^']*)': No such file or directory",
    r"cp: cannot create regular file '([^']*)': Not a directory",
];

fn match_(command: &Command) -> bool {
    PATTERNS.iter().any(|pattern| regex(pattern).is_match(command.output()))
}

fn get_new_command(command: &Command) -> Vec<String> {
    for pattern in PATTERNS {
        if let Some(file) = find_first(pattern, command.output()) {
            let dir = match file.rfind('/') {
                Some(slash) => &file[..slash],
                // No directory in the path: everything but the last char,
                // exactly what the original does.
                None => &file[..file.char_indices().next_back().map_or(0, |(index, _)| index)],
            };

            return vec![and_(&[&format!("mkdir -p {}", dir), &command.script])];
        }
    }
    vec![]
}

pub const RULE: Rule = Rule::new("no_such_file", match_, get_new_command);
