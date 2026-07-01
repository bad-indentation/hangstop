//! # HangStop
//! (c) 2026 by bad_indentation
//!
//! This module provides helper functions and structs for the command line
//! utility accessed in `main.rs`.

use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use regex::Regex;

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

    if forbidden
        .matches(|letter: char| !letter.is_alphabetic())
        .count()
        > 0
    {
        return Err("forbidden letters must only contain alphabetic characters.");
    }

    excluded.push_str(&forbidden.to_lowercase());

    let mut re = String::from("^");

    for letter in state.chars() {
        match letter {
            '?' => re.push_str(&format!("[^{excluded}]")),
            'a'..='z' => re.push(letter),
            _ => return Err("game state must only contain alphabetic characters or ?s."),
        }
    }

    re.push('$');
    Ok(re)
}

/// Prunes the wordlist and removes any words that do not fit the regex
fn prune_wordlist<T>(regex: String, wordlist: T) -> T
where
    T: IntoIterator<Item = String> + FromIterator<String>,
{
    let re = Regex::new(&regex).expect("regex should be valid.");

    wordlist
        .into_iter()
        .filter(|word| re.is_match(word))
        .collect()
}

/// Returns the number of bits of information associated with the
/// given probability `prob`
fn get_bits(prob: f32) -> f32 {
    f32::log2(1. / prob)
}

/// Returns a bitmask `word.len()` bits long.
///
/// A `1` in the bitmask indicates that the `letter` is present
/// in that space. A `0` indicates, _quelle surprise_, that it is
/// not present.
///
/// # Panics
/// This function panics if the `letter` is not alphabetic,
/// since this should have been checked earlier in the program
fn get_bitmask(letter: char, word: &str) -> u32 {
    let letter = match letter {
        'a'..='z' => letter,
        'A'..='Z' => letter.to_ascii_lowercase(),
        _ => panic!("`letter` must be an alphabetic character"),
    };

    let mut bitmask = 0;

    for space in word.chars() {
        bitmask <<= 1;
        if space == letter {
            bitmask += 1;
        }
    }

    bitmask
}

/// Small utility function. `p(x) * -log2(p(x))` for some `p(x)`
fn get_entropy_addend(occur: usize, total: usize) -> f32 {
    let prob = occur as f32 / total as f32;
    get_bits(prob) * prob
}

/// Returns the entropy, or the expected information to be gained,
/// of the given letter.
///
/// This is calculated using Shannon's entropy formula,
/// which is the sum `p(x) * -log2(p(x))` for all x
///
/// I'm sorry, I gave up on generics.
/// [ ] TODO: make this work on any iterable type
fn get_entropy(letter: char, wordlist: &HashSet<String>) -> f32 {
    let mut counter: HashMap<u32, usize> = HashMap::new();
    let mut total = 0;
    for word in wordlist {
        total += 1;
        counter
            .entry(get_bitmask(letter, word))
            .and_modify(|ct| *ct += 1)
            .or_insert(1);
    }

    counter
        .values()
        .map(|ct| get_entropy_addend(*ct, total))
        .sum()
}

/// This struct maps any given character to its associated entropy for easier
/// sorting and retrieval
struct LetterEntropy {
    letter: char,
    entropy: f32,
}

impl PartialEq for LetterEntropy {
    fn eq(&self, other: &Self) -> bool {
        self.entropy == other.entropy && self.letter == other.letter
    }
}

impl Eq for LetterEntropy {}

impl PartialOrd for LetterEntropy {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        // The swap between self and other is intentional so that the
        // list is sorted from greatest to least
        other.entropy.partial_cmp(&self.entropy)
    }
}

impl Ord for LetterEntropy {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        // The swap between self and other is intentional so that the
        // list is sorted from greatest to least
        other.entropy.total_cmp(&self.entropy)
    }
}

impl Display for LetterEntropy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:.3} bits", self.letter, self.entropy)
    }
}

/// Returns a sorted list of each letter matched to its respective entropy
fn get_sorted_entropies(remaining_letters: &str, wordlist: &HashSet<String>) -> Vec<LetterEntropy> {
    let mut entropies = Vec::new();

    for letter in remaining_letters.chars() {
        entropies.push(LetterEntropy {
            letter,
            entropy: get_entropy(letter, wordlist),
        });
    }

    entropies.sort();

    entropies
}

/// Returns all letters that can be guessed without repeating a
/// previous guess
fn get_guessable(state: &str, forbidden: &str) -> String {
    let mut guessable = String::new();

    for letter in ([state, forbidden].concat()).chars() {
        if letter.is_ascii_alphabetic() && !guessable.contains(letter) {
            guessable.push(letter);
        }
    }

    guessable
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_build_regex() {
        let re = build_regex("A???e", "krbg").unwrap();
        assert_eq!(re, "^a[^aekrbg][^aekrbg][^aekrbg]e$".to_string());
    }

    #[test]
    fn test_build_regex_with_invalid_characters() {
        let re = build_regex("*(*H)", "kjd");
        assert!(re.is_err());

        let re = build_regex("h?ll?", "(*&j)");
        assert!(re.is_err());
    }

    #[test]
    fn test_prune_wordlist() {
        let re = build_regex("abc??", "ghi").unwrap();
        let words: HashSet<String> = ["abcde", "abcef", "abcfg", "abcgh", "abcbe", "abc"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let expected: HashSet<String> = ["abcde", "abcef"].iter().map(|s| s.to_string()).collect();
        let filtered = prune_wordlist(re, words);

        assert_eq!(expected, filtered);
    }

    #[test]
    fn test_bits() {
        // Probably doesn't deserve a test, but oh well...
        assert_eq!(5.0, get_bits(1.0 / 32.0))
    }

    #[test]
    fn test_bitmask() {
        assert_eq!(0b00110, get_bitmask('l', "hello"));
        assert_eq!(0b010101, get_bitmask('a', "banana"));
    }

    #[test]
    fn test_entropy() {
        let words: HashSet<String> = ["hello", "lends", "flees", "stick"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(1.5, get_entropy('e', &words));
    }

    #[test]
    fn test_sorted_entropy() {
        let words: HashSet<String> = ["ab", "cb"].iter().map(|s| s.to_string()).collect();

        let mut sorted_letters = get_sorted_entropies("abcd", &words)
            .into_iter()
            .map(|letter| letter.letter);

        // Order does not matter for equally ranked letters.
        // 'a' or 'c' should have the most information (1 bit)
        let mut item = sorted_letters.next();
        assert!(item == Some('a') || item == Some('c'));

        item = sorted_letters.next();
        assert!(item == Some('a') || item == Some('c'));

        item = sorted_letters.next();
        assert!(item == Some('b') || item == Some('d'));

        item = sorted_letters.next();
        assert!(item == Some('b') || item == Some('d'));
    }

    #[test]
    fn test_guessable() {
        // Order of guessable letters does not matter
        let guessable: HashSet<char> =
            HashSet::from_iter((&get_guessable("?e??o", "tains")).chars());
        let expected = HashSet::from(['e', 'o', 't', 'a', 'i', 'n', 's']);

        assert_eq!(guessable, expected)
    }
}
