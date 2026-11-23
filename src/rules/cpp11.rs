use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["g++", "clang++"])
        && (command
            .output()
            .contains("This file requires compiler and library support for the ISO C++ 2011 standard.")
            || command.output().contains("-Wc++11-extensions"))
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![format!("{} -std=c++11", command.script)]
}

pub const RULE: Rule = Rule::new("cpp11", match_, get_new_command);
