//! Attempts to spellcheck and correct failed cd commands

use std::fs;
use std::path::{Path, PathBuf};

use super::cd_mkdir;
use super::prelude::*;

const MAX_ALLOWED_DIFF: f64 = 0.6;

/// Returns a list of the child directories of the given parent directory
fn get_sub_dirs(parent: &Path) -> Vec<String> {
    fs::read_dir(parent)
        .map(|entries| {
            entries
                .flatten()
                .filter(|child| child.path().is_dir())
                .map(|child| child.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// Attempt to rebuild the path string by spellchecking the directories.
/// If it fails (i.e. no directories are a close enough match), then it
/// defaults to the rules of cd_mkdir.
/// Change sensitivity by changing MAX_ALLOWED_DIFF. Default value is 0.6
fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| {
        let mut dest: Vec<&str> = command.script_parts()[1].split('/').collect();
        if dest.last() == Some(&"") {
            dest.pop();
        }
        let mut cwd = if dest.first() == Some(&"") {
            dest.remove(0);
            PathBuf::from("/")
        } else {
            std::env::current_dir().unwrap_or_default()
        };
        for directory in dest {
            if directory == "." {
                continue;
            } else if directory == ".." {
                cwd = cwd.parent().map_or_else(|| cwd.clone(), Path::to_path_buf);
                continue;
            }
            let best_matches = get_close_matches(directory, &get_sub_dirs(&cwd), MAX_ALLOWED_DIFF);
            match best_matches.first() {
                Some(best_match) => cwd.push(best_match),
                None => return cd_mkdir::get_new_command(command),
            }
        }
        vec![format!("cd \"{}\"", cwd.display())]
    })
}

pub const RULE: Rule = Rule::new("cd_correction", cd_mkdir::match_, get_new_command);
