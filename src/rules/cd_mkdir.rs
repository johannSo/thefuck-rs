use super::prelude::*;

pub(super) fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        let output = command.output().to_lowercase();
        for_app(command, &["cd"])
            && command.script.starts_with("cd ")
            && (output.contains("no such file or directory")
                || output.contains("cd: can't cd to")
                || output.contains("does not exist"))
    })
}

pub(super) fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| {
        let script = &command.script;
        match regex("^cd (.*)").captures(script) {
            Some(found) => {
                let repl = and_(&[&format!("mkdir -p {}", &found[1]), &format!("cd {}", &found[1])]);
                vec![format!("{}{}", repl, &script[found[0].len()..])]
            }
            None => vec![script.clone()],
        }
    })
}

pub const RULE: Rule = Rule::new("cd_mkdir", match_, get_new_command);
