//! This happens way too often
//!
//! When typing really fast cause I'm a 1337 H4X0R,
//! I often fuck up 'ls' and type 'sl'. No more!

use super::prelude::*;

fn match_(command: &Command) -> bool {
    command.script == "sl"
}

fn get_new_command(_command: &Command) -> Vec<String> {
    one("ls")
}

pub const RULE: Rule = Rule::new("sl_ls", match_, get_new_command);
