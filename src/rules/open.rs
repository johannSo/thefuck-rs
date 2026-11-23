//! Opens URL's in the default web browser
//!
//! Example:
//! ```text
//! > open github.com
//! The file ~/github.com does not exist.
//! Perhaps you meant 'http://github.com'?
//! ```

use super::prelude::*;

fn is_arg_url(command: &Command) -> bool {
    [".com", ".edu", ".info", ".io", ".ly", ".me", ".net", ".org", ".se", "www."]
        .iter()
        .any(|part| command.script.contains(part))
}

fn is_missing_file(output: &str) -> bool {
    output.starts_with("The file ") && output.ends_with(" does not exist.")
}

fn match_(command: &Command) -> bool {
    for_app(command, &["open", "xdg-open", "gnome-open", "kde-open"])
        && (is_arg_url(command) || is_missing_file(command.output().trim()))
}

fn get_new_command(command: &Command) -> Vec<String> {
    let output = command.output().trim();
    if is_arg_url(command) {
        vec![command.script.replace("open ", "open http://")]
    } else if is_missing_file(output) {
        let arg = command.script.split_once(' ').expect("the script has an argument").1;
        ["touch", "mkdir"]
            .iter()
            .map(|option| and_(&[&format!("{} {}", option, arg), &command.script]))
            .collect()
    } else {
        vec![]
    }
}

pub const RULE: Rule = Rule::new("open", match_, get_new_command);
