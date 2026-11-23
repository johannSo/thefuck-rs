//! The confusion in systemctl's param order is massive.

use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        // Catches "Unknown operation 'service'." when executing systemctl with
        // misordered arguments
        let cmd = command.script_parts();
        for_app(command, &["systemctl"])
            && command.output().contains("Unknown operation '")
            && cmd.len() - cmd.iter().position(|part| part == "systemctl").expect("systemctl is in the script") == 3
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| {
        let mut cmd = command.script_parts().to_vec();
        let len = cmd.len();
        cmd.swap(len - 1, len - 2);
        vec![cmd.join(" ")]
    })
}

pub const RULE: Rule = Rule::new("systemctl", match_, get_new_command);
