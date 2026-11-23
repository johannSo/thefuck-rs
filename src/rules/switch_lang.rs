use super::prelude::*;

const TARGET_LAYOUT: &str = "qwertyuiop[]asdfghjkl;'zxcvbnm,./QWERTYUIOP{}ASDFGHJKL:\"ZXCVBNM<>?";

const GREEK: &str = ";ςερτυθιοπ[]ασδφγηξκλ΄ζχψωβνμ,./:΅ΕΡΤΥΘΙΟΠ{}ΑΣΔΦΓΗΞΚΛ¨\"ΖΧΨΩΒΝΜ<>?";
const KOREAN: &str = "ㅂㅈㄷㄱㅅㅛㅕㅑㅐㅔ[]ㅁㄴㅇㄹㅎㅗㅓㅏㅣ;'ㅋㅌㅊㅍㅠㅜㅡ,./ㅃㅉㄸㄲㅆㅛㅕㅑㅒㅖ{}ㅁㄴㅇㄹㅎㅗㅓㅏㅣ:\"ㅋㅌㅊㅍㅠㅜㅡ<>?";

// any new keyboard layout must be appended
const SOURCE_LAYOUTS: &[&str] = &[
    "йцукенгшщзхъфывапролджэячсмитьбю.ЙЦУКЕНГШЩЗХЪФЫВАПРОЛДЖЭЯЧСМИТЬБЮ,",
    "йцукенгшщзхїфівапролджєячсмитьбю.ЙЦУКЕНГШЩЗХЇФІВАПРОЛДЖЄЯЧСМИТЬБЮ,",
    "ضصثقفغعهخحجچشسیبلاتنمکگظطزرذدپو./\u{652}\u{64c}\u{64d}\u{64b}\u{64f}\u{650}\u{64e}\u{651}][}{ؤئيإأآة»«:؛ك\u{653}ژ\u{670}\u{200c}\u{654}ء><؟",
    "/'קראטוןםפ][שדגכעיחלךף,זסבהנמצתץ.QWERTYUIOP{}ASDFGHJKL:\"ZXCVBNM<>?",
    GREEK,
    KOREAN,
];

/// Layouts that can't be switched by position of the chars.
const SOURCE_TO_TARGET: &[(&str, &[(char, char)])] = &[(
    GREEK,
    &[
        (';', 'q'), ('ς', 'w'), ('ε', 'e'), ('ρ', 'r'), ('τ', 't'), ('υ', 'y'),
        ('θ', 'u'), ('ι', 'i'), ('ο', 'o'), ('π', 'p'), ('[', '['), (']', ']'),
        ('α', 'a'), ('σ', 's'), ('δ', 'd'), ('φ', 'f'), ('γ', 'g'), ('η', 'h'),
        ('ξ', 'j'), ('κ', 'k'), ('λ', 'l'), ('΄', '\''), ('ζ', 'z'), ('χ', 'x'),
        ('ψ', 'c'), ('ω', 'v'), ('β', 'b'), ('ν', 'n'), ('μ', 'm'), (',', ','),
        ('.', '.'), ('/', '/'), (':', 'Q'), ('΅', 'W'), ('Ε', 'E'), ('Ρ', 'R'),
        ('Τ', 'T'), ('Υ', 'Y'), ('Θ', 'U'), ('Ι', 'I'), ('Ο', 'O'), ('Π', 'P'),
        ('{', '{'), ('}', '}'), ('Α', 'A'), ('Σ', 'S'), ('Δ', 'D'), ('Φ', 'F'),
        ('Γ', 'G'), ('Η', 'H'), ('Ξ', 'J'), ('Κ', 'K'), ('Λ', 'L'), ('¨', ':'),
        ('"', '"'), ('Ζ', 'Z'), ('Χ', 'X'), ('Ψ', 'C'), ('Ω', 'V'), ('Β', 'B'),
        ('Ν', 'N'), ('Μ', 'M'), ('<', '<'), ('>', '>'), ('?', '?'), ('ά', 'a'),
        ('έ', 'e'), ('ύ', 'y'), ('ί', 'i'), ('ό', 'o'), ('ή', 'h'), ('ώ', 'v'),
        ('Ά', 'A'), ('Έ', 'E'), ('Ύ', 'Y'), ('Ί', 'I'), ('Ό', 'O'), ('Ή', 'H'),
        ('Ώ', 'V'),
    ],
)];

// Lists used for decomposing korean letters.
const HEAD_LIST: &[char] = &['ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'];
const BODY_LIST: &[char] = &['ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ'];
const TAIL_LIST: &[char] = &[' ', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ', 'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ'];
const DOUBLE_LIST: &[char] = &['ㅘ', 'ㅙ', 'ㅚ', 'ㅝ', 'ㅞ', 'ㅟ', 'ㅢ', 'ㄳ', 'ㄵ', 'ㄶ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㅀ', 'ㅄ'];
const DOUBLE_MOD_LIST: &[&str] = &["ㅗㅏ", "ㅗㅐ", "ㅗㅣ", "ㅜㅓ", "ㅜㅔ", "ㅜㅣ", "ㅡㅣ", "ㄱㅅ", "ㄴㅈ", "ㄴㅎ", "ㄹㄱ", "ㄹㅁ", "ㄹㅂ", "ㄹㅅ", "ㄹㅌ", "ㄹㅎ", "ㅂㅅ"];

fn get_matched_layout(script: &str) -> Option<&'static str> {
    // don't use command.split_parts here because it handles quotes and
    // backslashes, which could be important
    SOURCE_LAYOUTS.iter().copied().find(|source_layout| {
        script.split(' ').all(|cmd_part| cmd_part.chars().all(|ch| source_layout.contains(ch) || "-_".contains(ch)))
    })
}

fn switch(ch: char, layout: &str) -> char {
    match layout.chars().position(|layout_ch| layout_ch == ch) {
        Some(position) => TARGET_LAYOUT.chars().nth(position).expect("layouts have the same size"),
        None => ch,
    }
}

fn switch_command(script: &str, layout: &str) -> String {
    // Layouts with different amount of characters than English
    if let Some((_, source_to_target)) = SOURCE_TO_TARGET.iter().find(|(source, _)| *source == layout) {
        return script
            .chars()
            .map(|ch| source_to_target.iter().find(|(source, _)| *source == ch).map_or(ch, |(_, target)| *target))
            .collect();
    }

    script.chars().map(|ch| switch(ch, layout)).collect()
}

fn is_korean(ch: char) -> bool {
    ('ㄱ'..='ㅎ').contains(&ch) || ('ㅏ'..='ㅣ').contains(&ch) || ('가'..='힣').contains(&ch)
}

fn decompose_korean(script: &str) -> String {
    fn change_double(ch: char, hg_str: &mut String) {
        match DOUBLE_LIST.iter().position(|double| *double == ch) {
            Some(position) => hg_str.push_str(DOUBLE_MOD_LIST[position]),
            None => hg_str.push(ch),
        }
    }

    let mut hg_str = String::new();
    for ch in script.chars() {
        if ('가'..='힣').contains(&ch) {
            let ord_ch = ch as usize - '가' as usize;
            let hd = ord_ch / 588;
            let bd = (ord_ch - 588 * hd) / 28;
            let tl = ord_ch - 588 * hd - 28 * bd;
            for ch in [HEAD_LIST[hd], BODY_LIST[bd], TAIL_LIST[tl]] {
                if ch != ' ' {
                    change_double(ch, &mut hg_str);
                }
            }
        } else {
            change_double(ch, &mut hg_str);
        }
    }
    hg_str
}

fn match_(command: &Command) -> bool {
    if !command.output().contains("not found") {
        return false;
    }
    if command.script.chars().any(is_korean) {
        return true;
    }

    get_matched_layout(&command.script)
        .is_some_and(|matched_layout| switch_command(&command.script, matched_layout) != get_alias())
}

fn get_new_command(command: &Command) -> Vec<String> {
    let script = if command.script.chars().any(is_korean) {
        decompose_korean(&command.script)
    } else {
        command.script.clone()
    };
    get_matched_layout(&script).map(|matched_layout| switch_command(&script, matched_layout)).into_iter().collect()
}

pub const RULE: Rule = Rule::new("switch_lang", match_, get_new_command);
