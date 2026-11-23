//! Fixes error for commands containing one or more occurrences of the shell
//! prompt symbol '$'.
//!
//! This usually happens when commands are copied from documentations
//! including them in their code blocks.
//!
//! Example:
//! ```text
//! > $ git clone https://github.com/nvbn/thefuck.git
//! bash: $: command not found...
//! ```

use super::prelude::*;

fn match_(command: &Command) -> bool {
    command.output().contains("$: command not found") && regex(r"^[\s]*\$ [\S]+").is_match(&command.script)
}

fn get_new_command(command: &Command) -> Vec<String> {
    vec![command.script.trim_start_matches(['$', ' ']).to_string()]
}

pub const RULE: Rule = Rule::new("remove_shell_prompt_literal", match_, get_new_command);
