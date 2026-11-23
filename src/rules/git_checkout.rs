use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.output().contains("did not match any file(s) known to git")
            && !command.output().contains("Did you forget to 'git add'?")
    })
}

fn parse_branches(output: &str) -> Vec<String> {
    let mut branches = Vec::new();
    for line in output.split_inclusive('\n') {
        if line.contains("->") {
            // Remote HEAD like b'  remotes/origin/HEAD -> origin/master'
            continue;
        }
        let mut line = line.to_string();
        if line.starts_with('*') {
            line = line.split(' ').nth(1).unwrap_or("").to_string();
        }
        if line.trim().starts_with("remotes/") {
            line = line.split('/').skip(2).collect::<Vec<_>>().join("/");
        }
        branches.push(line.trim().to_string());
    }
    branches
}

fn get_branches() -> Vec<String> {
    parse_branches(&run("git", &["branch", "-a", "--no-color", "--no-column"]).stdout)
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        let missing_file = &findall(
            r"error: pathspec '([^']*)' did not match any file\(s\) known to git",
            command.output(),
        )[0];
        let closest_branch = get_closest(missing_file, &get_branches(), 0.6, false);

        let mut new_commands = Vec::new();

        if let Some(closest_branch) = closest_branch {
            new_commands.push(replace_argument(&command.script, missing_file, &closest_branch));
        }
        if command.script_parts()[1] == "checkout" {
            new_commands.push(replace_argument(&command.script, "checkout", "checkout -b"));
        }

        if new_commands.is_empty() {
            new_commands.push(and_(&[&format!("git branch {}", missing_file), &command.script]));
        }

        new_commands
    })
}

pub const RULE: Rule = Rule::new("git_checkout", match_, get_new_command);

#[cfg(test)]
mod tests {
    use super::parse_branches;

    #[test]
    fn parses_branches() {
        assert_eq!(parse_branches(""), Vec::<String>::new());
        assert_eq!(parse_branches("* master\n"), ["master"]);
        assert_eq!(parse_branches("  remotes/origin/master\n"), ["master"]);
        assert_eq!(parse_branches("  remotes/origin/test/1\n"), ["test/1"]);
        assert_eq!(parse_branches("  just-another-branch\n"), ["just-another-branch"]);
        assert_eq!(parse_branches("* master\n  just-another-branch\n"), ["master", "just-another-branch"]);
        assert_eq!(
            parse_branches("* master\n  remotes/origin/master\n  just-another-branch\n"),
            ["master", "master", "just-another-branch"]
        );
        assert_eq!(
            parse_branches("  remotes/origin/HEAD -> origin/master\n  remotes/origin/foo\n"),
            ["foo"]
        );
    }
}
