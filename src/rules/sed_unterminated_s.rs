use super::prelude::*;
use crate::shlex;

fn match_(command: &Command) -> bool {
    for_app(command, &["sed"]) && command.output().contains("unterminated `s' command")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let mut script = shlex::split(&command.script).expect("the script can be split");

    for e in &mut script {
        if (e.starts_with("s/") || e.starts_with("-es/")) && !e.ends_with('/') {
            e.push('/');
        }
    }

    vec![script.iter().map(|part| shell().quote(part)).collect::<Vec<_>>().join(" ")]
}

pub const RULE: Rule = Rule::new("sed_unterminated_s", match_, get_new_command);
