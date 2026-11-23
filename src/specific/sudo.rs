use crate::types::Command;

/// Result of a rule function that can be adapted to a call with `sudo`.
pub trait SudoResult {
    fn with_sudo(self) -> Self;
}

impl SudoResult for bool {
    fn with_sudo(self) -> Self {
        self
    }
}

impl SudoResult for Vec<String> {
    fn with_sudo(self) -> Self {
        self.into_iter()
            .map(|script| format!("sudo {}", script))
            .collect()
    }
}

/// Removes sudo before calling fn and adds it after.
pub fn sudo_support<T: SudoResult>(command: &Command, f: impl FnOnce(&Command) -> T) -> T {
    match command.script.strip_prefix("sudo ") {
        Some(script) => f(&command.with_script(script)).with_sudo(),
        None => f(command),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_sudo() {
        let script = |command: &Command| vec![format!("{} --fixed", command.script)];
        let is_ls = |command: &Command| command.script == "ls";
        assert_eq!(
            sudo_support(&Command::new("sudo ls", Some("")), script),
            ["sudo ls --fixed"]
        );
        assert_eq!(
            sudo_support(&Command::new("ls", Some("")), script),
            ["ls --fixed"]
        );
        assert!(sudo_support(&Command::new("sudo ls", Some("")), is_ls));
        assert!(sudo_support(&Command::new("ls", Some("")), is_ls));
        assert!(!sudo_support(&Command::new("sudo cd", Some("")), is_ls));
    }
}
