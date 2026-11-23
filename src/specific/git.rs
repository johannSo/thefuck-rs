use regex::NoExpand;

use crate::shells::shell;
use crate::types::Command;
use crate::utils::{is_app, regex};

/// Resolves git aliases and supports testing for both git and hub.
///
/// Returns the default value (no match, no new commands) when the command
/// isn't a call of git.
pub fn git_support<T: Default>(command: &Command, f: impl FnOnce(&Command) -> T) -> T {
    // supports GitHub's `hub` command
    // which is recommended to be used with `alias git=hub`
    // but at this point, shell aliases have already been resolved
    if !is_app(command, &["git", "hub"], 0) {
        return T::default();
    }

    // perform git aliases expansion
    let expansion = command
        .output
        .as_deref()
        .filter(|output| output.contains("trace: alias expansion:"))
        .and_then(|output| regex("trace: alias expansion: ([^ ]*) => ([^\n]*)").captures(output));
    if let Some(search) = expansion {
        let alias = &search[1];

        // by default git quotes everything, for example:
        //     'commit' '--amend'
        // which is surprising and does not allow to easily test for
        // eg. 'git commit'
        let expansion = shell()
            .split_command(&search[2])
            .iter()
            .map(|part| shell().quote(part))
            .collect::<Vec<_>>()
            .join(" ");
        let new_script =
            regex(&format!(r"\b{}\b", alias)).replace_all(&command.script, NoExpand(&expansion));

        return f(&command.with_script(new_script));
    }

    f(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_git_aliases() {
        let output =
            "19:22:36.299340 git.c:282   trace: alias expansion: com => 'commit' '--verbose'";
        let script = |command: &Command| command.script.clone();
        assert_eq!(
            git_support(&Command::new("git com file", Some(output)), script),
            "git commit --verbose file"
        );
        assert_eq!(
            git_support(&Command::new("git com file", Some("")), script),
            "git com file"
        );
        assert_eq!(git_support(&Command::new("hub br", None), script), "hub br");
        assert_eq!(git_support(&Command::new("ls", Some(output)), script), "");
    }

    #[test]
    fn supports_only_git_and_hub() {
        let always = |_: &Command| true;
        assert!(git_support(&Command::new("git pull", Some("")), always));
        assert!(git_support(&Command::new("hub pull", Some("")), always));
        assert!(git_support(
            &Command::new("git push --set-upstream origin foobar", Some("")),
            always
        ));
        assert!(!git_support(&Command::new("ghub pull", Some("")), always));
        assert!(!git_support(&Command::new("cat git", Some("")), always));
    }
}
