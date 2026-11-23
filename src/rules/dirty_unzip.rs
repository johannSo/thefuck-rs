use std::fs;

use super::prelude::*;

/// Returns names of the files in the zip archive.
fn archive_names(data: &[u8]) -> Option<Vec<String>> {
    const END_OF_CENTRAL_DIRECTORY: &[u8] = b"PK\x05\x06";
    const CENTRAL_DIRECTORY_ENTRY: &[u8] = b"PK\x01\x02";
    let u16_at = |pos: usize| Some(u16::from_le_bytes(data.get(pos..pos + 2)?.try_into().ok()?) as usize);
    let u32_at = |pos: usize| Some(u32::from_le_bytes(data.get(pos..pos + 4)?.try_into().ok()?) as usize);

    // The record is at the end of the file, followed only by a comment.
    let end = (0..=data.len().checked_sub(22)?)
        .rev()
        .take(22 + usize::from(u16::MAX))
        .find(|pos| data[*pos..].starts_with(END_OF_CENTRAL_DIRECTORY))?;
    let entries = u16_at(end + 10)?;
    let size = u32_at(end + 12)?;

    let mut names = Vec::new();
    // The archive can be prepended with other data, so the offset stored in
    // the record isn't used.
    let mut pos = end.checked_sub(size)?;
    for _ in 0..entries {
        if !data.get(pos..)?.starts_with(CENTRAL_DIRECTORY_ENTRY) {
            return None;
        }
        let name_len = u16_at(pos + 28)?;
        let extra_len = u16_at(pos + 30)?;
        let comment_len = u16_at(pos + 32)?;
        names.push(String::from_utf8_lossy(data.get(pos + 46..pos + 46 + name_len)?).into_owned());
        pos += 46 + name_len + extra_len + comment_len;
    }
    Some(names)
}

fn is_bad_zip(file: &str) -> bool {
    fs::read(file).ok().and_then(|data| archive_names(&data)).is_some_and(|names| names.len() > 1)
}

fn zip_file(command: &Command) -> Option<String> {
    // unzip works that way:
    // unzip [-flags] file[.zip] [file(s) ...] [-x file(s) ...]
    //                ^          ^ files to unzip from the archive
    //                archive to unzip
    command.script_parts()[1..].iter().find(|c| !c.starts_with('-')).map(|c| {
        if c.ends_with(".zip") {
            c.clone()
        } else {
            format!("{}.zip", c)
        }
    })
}

fn match_(command: &Command) -> bool {
    if !for_app(command, &["unzip"]) || command.script.contains("-d") {
        return false;
    }

    zip_file(command).is_some_and(|zip_file| is_bad_zip(&zip_file))
}

fn get_new_command(command: &Command) -> Vec<String> {
    let Some(zip_file) = zip_file(command) else { return vec![] };
    vec![format!("{} -d {}", command.script, shell().quote(&zip_file[..zip_file.len() - 4]))]
}

/// Removes the files the archive was extracted to in the current directory.
fn side_effect(old_cmd: &Command, _command: &str) {
    let Some(names) = zip_file(old_cmd).and_then(|file| fs::read(file).ok()).and_then(|data| archive_names(&data))
    else {
        return;
    };
    remove_extracted_files(names.iter().map(String::as_str));
}

pub const RULE: Rule = Rule {
    side_effect: Some(side_effect),
    requires_output: false,
    ..Rule::new("dirty_unzip", match_, get_new_command)
};

#[cfg(test)]
mod tests {
    use super::archive_names;

    /// Builds an archive of empty files: only the parts needed for listing.
    fn archive(names: &[&str], prefix: &[u8]) -> Vec<u8> {
        let mut data = prefix.to_vec();
        let mut directory = Vec::new();
        for name in names {
            let offset = data.len() as u32;
            data.extend(b"PK\x03\x04");
            data.extend([0; 22]);
            data.extend((name.len() as u16).to_le_bytes());
            data.extend([0; 2]);
            data.extend(name.as_bytes());

            directory.extend(b"PK\x01\x02");
            directory.extend([0; 24]);
            directory.extend((name.len() as u16).to_le_bytes());
            directory.extend([0; 12]);
            directory.extend(offset.to_le_bytes());
            directory.extend(name.as_bytes());
        }
        let directory_offset = data.len() as u32;
        data.extend(&directory);
        data.extend(b"PK\x05\x06");
        data.extend([0; 4]);
        data.extend((names.len() as u16).to_le_bytes());
        data.extend((names.len() as u16).to_le_bytes());
        data.extend((directory.len() as u32).to_le_bytes());
        data.extend(directory_offset.to_le_bytes());
        data.extend([0; 2]);
        data
    }

    #[test]
    fn lists_files_of_archive() {
        assert_eq!(archive_names(&archive(&["a", "b/c", "d é"], b"")).unwrap(), ["a", "b/c", "d é"]);
        assert_eq!(archive_names(&archive(&["a"], b"#!/bin/sh\n")).unwrap(), ["a"]);
        assert!(archive_names(&archive(&[], b"")).unwrap().is_empty());
        assert_eq!(archive_names(b"not a zip archive, just some text"), None);
        assert_eq!(archive_names(b""), None);
    }
}
