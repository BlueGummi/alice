use std::process; 
use colorized::*;


pub fn remove_comments(f_contents: &mut String) -> String {
    let mut result = String::new();

    for line in f_contents.lines() {
        if let Some(comment_loc) = line.find(';') {
            result.push_str(&line[..comment_loc]); // append part before comment
        } else {
            result.push_str(line); // append whole line if no comment
        }
        result.push('\n'); // add newline
    }

    *f_contents = result.trim_end().to_string(); // update original string
    result.trim_end().to_string()
}

pub fn delete_last_letter(s: &str) -> &str {
    if !s.is_empty() {
        let last_char = s.chars().last().unwrap();
        if last_char.is_alphabetic() {
            return &s[..s.len() - 1]; // return slice excluding last character
        }
    }
    s // return original string if empty or last char not a letter
}

pub fn append_line_numbers(input: &str) -> String {
    input.lines()
        .enumerate()
        .map(|(i, line)| format!("{} {}", i + 1, line))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn neg_num_err(instruction: &str) {
    eprintln!(
        "{}{}{}",
        "ERROR, ".color(Colors::RedFg),
        instruction.color(Colors::YellowFg),
        " WILL RESULT IN NEGATIVE NUMBER.\nTERMINATING.".color(Colors::RedFg)
    );
    process::exit(0);
}

pub fn err_print(error: String) {
    eprintln!(
        "{}{}",
        "ERROR, ".color(Colors::RedFg),
        error.color(Colors::RedFg)
    );
    process::exit(0);
}

pub fn letter_to_integer(letter: char) -> Option<u8> {
    if letter.is_ascii_lowercase() {
        Some(letter as u8 - b'a')
    } else if letter.is_ascii_uppercase() {
        Some(letter as u8 - b'A')
    } else {
        None
    }
}

pub fn integer_to_letter(n: usize) -> char {
    if n < 26 {
        (n as u8 + b'a') as char
    } else {
        err_print("value passed to integer_to_letter was too large.".to_string());
        process::exit(0);
    }
}

pub fn has_single_letter(s: &str) -> bool {
    s.len() == 1 && s.chars().next().unwrap().is_alphabetic()
}

pub fn has_b_with_num(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut found_b = false;
    for &byte in bytes {
        if found_b {
            if byte.is_ascii_digit() {
                return true;
            } else {
                found_b = false; // reset if not a digit
            }
        }
        if byte == b'b' || byte == b'B' {
            found_b = true;
        }
    }
    false
}