use super::prelude::*;

const NO_COMMAND: &str = "Error: No such command";
const NO_WEBSITE: &str = "hostscli.errors.WebsiteImportError";

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        for_app(command, &["hostscli"])
            && [NO_COMMAND, NO_WEBSITE].iter().any(|error| command.output().contains(error))
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    sudo_support(command, |command| {
        if command.output().contains(NO_WEBSITE) {
            return one("hostscli websites");
        }

        let misspelled_command = regex(r#"Error: No such command ".*""#)
            .find(command.output())
            .expect("the error names the command")
            .as_str();
        let commands = ["block", "unblock", "websites", "block_all", "unblock_all"];
        replace_command(command, misspelled_command, &commands)
    })
}

pub const RULE: Rule = Rule::new("hostscli", match_, get_new_command);
