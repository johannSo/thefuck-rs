use std::path::Path;

use super::prelude::*;

const PATH_TO_SCM: &[(&str, &str)] = &[(".git", "git"), (".hg", "hg")];

const WRONG_SCM_PATTERNS: &[(&str, &str)] =
    &[("git", "fatal: Not a git repository"), ("hg", "abort: no repository found")];

fn get_actual_scm() -> Option<&'static str> {
    PATH_TO_SCM.iter().find(|(path, _)| Path::new(path).is_dir()).map(|(_, scm)| *scm)
}

fn match_(command: &Command) -> bool {
    if !for_app(command, &["git", "hg"]) {
        return false;
    }
    let scm = command.script_parts()[0].as_str();
    let pattern = WRONG_SCM_PATTERNS.iter().find(|(name, _)| *name == scm).expect("known scm").1;

    command.output().contains(pattern) && get_actual_scm().is_some()
}

fn get_new_command(command: &Command) -> Vec<String> {
    let Some(scm) = get_actual_scm() else { return vec![] };
    let mut parts = vec![scm.to_string()];
    parts.extend_from_slice(&command.script_parts()[1..]);
    vec![parts.join(" ")]
}

pub const RULE: Rule = Rule::new("scm_correction", match_, get_new_command);
