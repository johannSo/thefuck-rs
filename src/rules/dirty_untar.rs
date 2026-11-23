use super::prelude::*;

const TAR_EXTENSIONS: &[&str] = &[
    ".tar", ".tar.Z", ".tar.bz2", ".tar.gz", ".tar.lz", ".tar.lzma", ".tar.xz", ".taz", ".tb2", ".tbz", ".tbz2",
    ".tgz", ".tlz", ".txz", ".tz",
];

fn is_tar_extract(cmd: &str) -> bool {
    if cmd.contains("--extract") {
        return true;
    }

    let cmd: Vec<&str> = cmd.split_whitespace().collect();

    cmd.len() > 1 && cmd[1].contains('x')
}

/// Returns the archive and its name without the extension.
fn tar_file(cmd: &[String]) -> Option<(&str, &str)> {
    for c in cmd {
        for ext in TAR_EXTENSIONS {
            if let Some(name) = c.strip_suffix(ext) {
                return Some((c, name));
            }
        }
    }
    None
}

fn match_(command: &Command) -> bool {
    for_app(command, &["tar"])
        && !command.script.contains("-C")
        && is_tar_extract(&command.script)
        && tar_file(command.script_parts()).is_some()
}

fn get_new_command(command: &Command) -> Vec<String> {
    let Some((_, name)) = tar_file(command.script_parts()) else { return vec![] };
    let dir = shell().quote(name);

    vec![and_(&[&format!("mkdir -p {}", dir), &format!("{} -C {}", command.script, dir)])]
}

/// Removes the files the archive was extracted to in the current directory.
fn side_effect(old_cmd: &Command, _command: &str) {
    let Some((archive, _)) = tar_file(old_cmd.script_parts()) else { return };
    let listing = run("tar", &["-tf", archive]);
    remove_extracted_files(listing.stdout.lines());
}

pub const RULE: Rule = Rule { side_effect: Some(side_effect), ..Rule::new("dirty_untar", match_, get_new_command) };
