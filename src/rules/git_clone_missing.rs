//! Rule: git_clone_missing
//!
//! Correct missing `git clone` command when pasting a git URL
//!
//! ```sh
//! >>> https://github.com/nvbn/thefuck.git
//! git clone https://github.com/nvbn/thefuck.git
//! ```
//!
//! Author: Miguel Guthridge

use super::prelude::*;
use crate::specific::url::urlparse;

fn match_(command: &Command) -> bool {
    // We want it to be a URL by itself
    if command.script_parts().len() != 1 {
        return false;
    }
    // Ensure we got the error we expected
    let output = command.output();
    if which(&command.script_parts()[0]).is_some()
        || !(output.contains("No such file or directory")
            || output.contains("not found")
            || output.contains("is not recognised as"))
    {
        return false;
    }
    let url = urlparse(&command.script, "ssh");
    // HTTP URLs need a network address
    if url.netloc.is_empty() && url.scheme != "ssh" {
        return false;
    }
    // SSH needs a username and a splitter between the path
    if url.scheme == "ssh" && !(command.script.contains('@') && command.script.contains(':')) {
        return false;
    }
    ["http", "https", "ssh"].contains(&url.scheme.as_str())
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![format!("git clone {}", command.script)]
}

pub const RULE: Rule = Rule::new("git_clone_missing", match_, get_new_command);
