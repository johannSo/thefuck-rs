//! Interactive selection of the fixed command.

use crate::conf::settings;
use crate::consts::{Action, Key};
use crate::corrector::organize_commands;
use crate::logs;
use crate::system::get_key;
use crate::types::CorrectedCommand;
use crate::utils::get_alias;

/// Waits for a key that means something and returns its action.
pub fn read_action() -> Action {
    loop {
        // Handle arrows, j/k (qwerty), and n/e (colemak)
        match get_key() {
            Key::Up | Key::CtrlN | Key::Char('k' | 'e') => return Action::Previous,
            Key::Down | Key::CtrlP | Key::Char('j' | 'n') => return Action::Next,
            Key::CtrlC | Key::Char('q') => return Action::Abort,
            Key::Char('\n' | '\r') => return Action::Select,
            Key::Char(_) => {}
        }
    }
}

/// Helper for selecting rule from rules list.
pub struct CommandSelector<I: Iterator<Item = CorrectedCommand>> {
    commands: Vec<CorrectedCommand>,
    /// Not yet evaluated commands, they are needed only when the user isn't
    /// satisfied with the first one.
    remaining: Option<I>,
    index: usize,
}

impl<I: Iterator<Item = CorrectedCommand>> CommandSelector<I> {
    /// Returns `None` when no rule matched.
    pub fn new(mut commands: I) -> Option<Self> {
        let first = commands.next()?;
        Some(CommandSelector {
            commands: vec![first],
            remaining: Some(commands),
            index: 0,
        })
    }

    fn realise(&mut self) {
        if let Some(remaining) = self.remaining.take() {
            let rest = organize_commands(&self.commands[0], remaining);
            self.commands.extend(rest);
        }
    }

    pub fn next(&mut self) {
        self.realise();
        self.index = (self.index + 1) % self.commands.len();
    }

    pub fn previous(&mut self) {
        self.realise();
        self.index = (self.index + self.commands.len() - 1) % self.commands.len();
    }

    pub fn value(&self) -> &CorrectedCommand {
        &self.commands[self.index]
    }
}

/// Returns:
///
///  - the first command when confirmation disabled;
///  - `None` when ctrl+c pressed;
///  - selected command.
pub fn select_command(
    corrected_commands: impl Iterator<Item = CorrectedCommand>,
    mut read_action: impl FnMut() -> Action,
) -> Option<CorrectedCommand> {
    let Some(mut selector) = CommandSelector::new(corrected_commands) else {
        logs::failed(if get_alias() == "fuck" {
            "No fucks given"
        } else {
            "Nothing found"
        });
        return None;
    };

    if !settings().require_confirmation {
        logs::show_corrected_command(selector.value());
        return Some(selector.value().clone());
    }

    logs::confirm_text(selector.value());

    loop {
        match read_action() {
            Action::Select => {
                eprintln!();
                return Some(selector.value().clone());
            }
            Action::Abort => {
                logs::failed("\nAborted");
                return None;
            }
            Action::Previous => {
                selector.previous();
                logs::confirm_text(selector.value());
            }
            Action::Next => {
                selector.next();
                logs::confirm_text(selector.value());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commands() -> impl Iterator<Item = CorrectedCommand> {
        [("ls", 100), ("cd", 300), ("ls", 500), ("pwd", 200)]
            .into_iter()
            .map(|(script, priority)| CorrectedCommand {
                script: script.to_string(),
                side_effect: None,
                priority,
            })
    }

    fn select(actions: &[Action]) -> Option<String> {
        let mut actions = actions.iter().copied();
        select_command(commands(), || actions.next().unwrap()).map(|command| command.script)
    }

    #[test]
    fn selector_cycles_through_sorted_commands() {
        let mut selector = CommandSelector::new(commands()).unwrap();
        assert_eq!(selector.value().script, "ls");
        selector.next();
        assert_eq!(selector.value().script, "pwd");
        selector.next();
        assert_eq!(selector.value().script, "cd");
        selector.next();
        assert_eq!(selector.value().script, "ls");
        selector.previous();
        assert_eq!(selector.value().script, "cd");
    }

    #[test]
    fn selects_commands_with_actions() {
        assert_eq!(select(&[Action::Select]).unwrap(), "ls");
        assert_eq!(select(&[Action::Next, Action::Select]).unwrap(), "pwd");
        assert_eq!(select(&[Action::Previous, Action::Select]).unwrap(), "cd");
        assert_eq!(select(&[Action::Next, Action::Abort]), None);
        assert_eq!(select_command(std::iter::empty(), || Action::Select), None);
    }
}
