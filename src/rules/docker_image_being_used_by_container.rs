use super::prelude::*;

/// Matches a command's output with docker's output
/// warning you that you need to remove a container before removing an image.
fn match_(command: &Command) -> bool {
    for_app(command, &["docker"]) && command.output().contains("image is being used by running container")
}

/// Prepends docker container rm -f {container ID} to
/// the previous docker image rm {image ID} command
fn get_new_command(command: &Command) -> Vec<String> {
    let container_id = command.output().trim().split(' ').next_back().unwrap_or("");
    vec![and_(&[&format!("docker container rm -f {}", container_id), &command.script])]
}

pub const RULE: Rule = Rule::new("docker_image_being_used_by_container", match_, get_new_command);
