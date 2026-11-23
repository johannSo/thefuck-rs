use super::prelude::*;

fn match_(command: &Command) -> bool {
    git_support(command, |command| {
        command.script_parts().iter().any(|part| part == "push")
            && command.output().contains("git push --set-upstream")
    })
}

fn get_upstream_option_index(command_parts: &[String]) -> Option<usize> {
    command_parts
        .iter()
        .position(|part| part == "--set-upstream")
        .or_else(|| command_parts.iter().position(|part| part == "-u"))
}

fn get_new_command(command: &Command) -> Vec<String> {
    git_support(command, |command| {
        // If --set-upstream or -u are passed, remove it and its argument. This is
        // because the remaining arguments are concatenated onto the command suggested
        // by git, which includes --set-upstream and its argument
        let mut command_parts = command.script_parts().to_vec();
        let upstream_option_index = get_upstream_option_index(&command_parts);

        if let Some(upstream_option_index) = upstream_option_index {
            command_parts.remove(upstream_option_index);

            // In case of `git push -u` we don't have next argument:
            if command_parts.len() > upstream_option_index {
                command_parts.remove(upstream_option_index);
            }
        } else {
            // the only non-qualified permitted options are the repository and refspec; git's
            // suggestion include them, so they won't be lost, but would be duplicated otherwise.
            let push_idx = command_parts.iter().position(|part| part == "push").expect("push is in the script") + 1;
            while command_parts.len() > push_idx && !command_parts[command_parts.len() - 1].starts_with('-') {
                command_parts.pop();
            }
        }

        let suggestions = findall(r"git push (.*)", command.output());
        let arguments = suggestions[suggestions.len() - 1].replace('\'', r"\'");
        vec![replace_argument(&command_parts.join(" "), "push", &format!("push {}", arguments.trim()))]
    })
}

pub const RULE: Rule = Rule::new("git_push", match_, get_new_command);
