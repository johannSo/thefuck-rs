use super::prelude::*;

fn match_(command: &Command) -> bool {
    for_app(command, &["tsuru"])
        && command.output().contains(" is not a tsuru command. See \"tsuru help\".")
        && command.output().contains("\nDid you mean?\n\t")
}

fn get_new_command(command: &Command) -> Vec<String> {
    let broken_cmd = &findall(r#"tsuru: "([^"]*)" is not a tsuru command"#, command.output())[0];

    replace_command(command, broken_cmd, &get_all_matched_commands(command.output(), &["Did you mean"]))
}

pub const RULE: Rule = Rule::new("tsuru_not_command", match_, get_new_command);
