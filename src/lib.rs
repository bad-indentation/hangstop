//! # HangStop
//! (c) 2026 by bad_indentation
//!
//! This module provides helper functions and structs for the command line
//! utility accessed in `main.rs`. 

/// Constructs a regular expression, as an owned String, given the game state
/// and the incorrectly guessed characters. If `state` contains any characters
/// other than `?` and ASCII letters or `forbidden` contains any characters 
/// other than ASCII letters, an `Err()` variant will be returned.
///
/// # Arguments
/// `state`: The correctly guessed letters and blanks (represented by `?`),
/// in the order that they would appear in the partially revealed answer. 
/// For instance, in a game of Hangman, if the secret word is `hello` and
/// the other player has guessed `h` and `l`, the resulting state would look
/// like `h?ll?`. Note that these letters are also excluded by the regex
/// because the rules of Hangman dictate that once a letter is guessed, 
/// all occurences of that letter must be uncovered.
///
/// `forbidden`: Any incorrectly guessed letters.
///
fn build_regex(state: &str, forbidden: &str) -> Result<String, &'static str> {
    let state = state.to_lowercase();
    let forbidden = forbidden.to_lowercase();

    let mut excluded = state.replace("?", "");

    if forbidden.matches(|letter: char| !letter.is_alphabetic()).count() > 0 {
        return Err("forbidden letters must only contain alphabetic characters.");
    }

    excluded.push_str(&forbidden.to_lowercase());

    let mut re = String::new();

    for letter in state.chars() {
        match letter {
            '?' => re.push_str(&format!("[a-z^{excluded}]")),
            'a'..='z' => re.push(letter),
            _ => return Err("game state must only contain alphabetic characters or ?s.")
        }
    }

    Ok(re)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_regex() {
        let re = build_regex("A???e", "krbg").unwrap();
        assert_eq!(re, "a[a-z^aekrbg][a-z^aekrbg][a-z^aekrbg]e".to_string());
    }

    #[test]
    fn test_build_regex_with_invalid_characters() {
        let re = build_regex("*(*H)", "kjd");
        assert!(re.is_err());

        let re = build_regex("h?ll?", "(*&j)");
        assert!(re.is_err());
    }
}
