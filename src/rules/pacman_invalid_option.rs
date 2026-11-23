use super::prelude::*;

fn match_(command: &Command) -> bool {
    sudo_support(command, |command| {
        for_app(command, &["pacman"])
            && command.output().starts_with("error: invalid option '-")
            && "surqfdvt".chars().any(|option| command.script.contains(&format!(" -{}", option)))
    })
}

fn get_new_command(command: &Command) -> Vec<String> {
    let option = regex(r" -[dfqrstuv]").find(&command.script).expect("the script has the option").as_str();
    vec![command.script.replace(option, &option.to_uppercase())]
}

pub const RULE: Rule = Rule::new("pacman_invalid_option", match_, get_new_command);
