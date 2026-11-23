use super::prelude::*;

fn extract_possibilities(command: &Command) -> Vec<String> {
    let possib = findall(r"\n\(did you mean one of ([^\?]+)\?\)", command.output());
    if let Some(possib) = possib.first() {
        return possib.split(", ").map(String::from).collect();
    }
    let possib = findall(r"\n    ([^$]+)\z", command.output());
    if let Some(possib) = possib.first() {
        return possib.split(' ').map(String::from).collect();
    }
    possib
}

fn match_(command: &Command) -> bool {
    let output = command.output();
    for_app(command, &["hg"])
        && ((output.contains("hg: unknown command") && output.contains("(did you mean one of "))
            || (output.contains("hg: command '") && output.contains("' is ambiguous:")))
}

fn get_new_command(command: &Command) -> Vec<String> {
    let mut script = command.script_parts().to_vec();
    let possibilities = extract_possibilities(command);
    let Some(closest) = get_closest(&script[1], &possibilities, 0.6, true) else { return vec![] };
    script[1] = closest;
    vec![script.join(" ")]
}

pub const RULE: Rule = Rule::new("mercurial", match_, get_new_command);
