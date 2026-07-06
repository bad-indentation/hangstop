//! # HangStop
//! (c) 2026 by bad_indentation
//!
//! This module provides helper functions and structs for the command line
//! utility accessed in `main.rs`.

use std::cmp;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Display;
use std::iter::zip;

// use regex::Regex;
use clap::Parser;

pub const ALL_WORDS: &str = include_str!("../public/wordlist.txt");

/// Contains the game state (the partially revealed word), the forbidden letters,
/// and generates the letters to be exclude
/// and generates the letters to be excluded
pub struct GameData<'a> {
    pub state: &'a str,
    pub forbidden: &'a str,
    pub all_excluded: HashSet<char>,
}

impl GameData<'_> {
    pub fn new<'a>(state: &'a str, forbidden: &'a str) -> GameData<'a> {
        let all_excluded =
            HashSet::from_iter([state, forbidden].concat().chars().filter(|c| *c != '?'));

        GameData {
            state,
            forbidden,
            all_excluded,
        }
    }
}

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
pub fn build_regex(state: &str, forbidden: &str) -> Result<String, &'static str> {
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
            '?' =>  if !excluded.is_empty() {
                re.push_str(&format!("[^{excluded}]")); 
            } else { re.push('.'); },
            'a'..='z' => re.push(letter),
            _ => return Err("game state must only contain alphabetic characters or ?s."),
        }
    }

    re.push('$');
    Ok(re)
}

/// Returns `true` if the word contains all of the letters uncovered in `state`,
/// in the correct locations, and does not contain any of `all_excluded` in
/// the covered locations.
fn matches_constraints(word: &str, data: &GameData) -> bool {
    if word.len() != data.state.len() {
        return false;
    }

    for (word_letter, state_char) in zip(word.chars(), data.state.chars()) {
        if state_char != '?' && word_letter != state_char {
            return false;
        }

        if state_char == '?' && data.all_excluded.contains(&word_letter) {
            return false;
        }
    }

    true
}

/// Prunes the wordlist and removes any words that do not fit the regex
pub fn prune_wordlist<T>(wordlist: T, data: &GameData) -> T
where
    T: IntoIterator<Item = String> + FromIterator<String>,
{
    eprintln!("pruning wordlist");
    wordlist
        .into_iter()
        .filter(|word| matches_constraints(word, &data))
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
    eprintln!("getting entropy for {letter}");

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
pub struct LetterEntropy {
    letter: char,
    entropy: f32,
}

impl LetterEntropy {
    /// Accessor method. Returns the associated letter
    pub fn get_letter(&self) -> char {
        self.letter
    }

    /// Accessor method. Returns the associated entropy
    pub fn get_entropy(&self) -> f32 {
        self.entropy
    }
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
pub fn get_sorted_entropies(remaining_letters: &str, wordlist: &HashSet<String>) -> Vec<LetterEntropy> {
    eprintln!("sorting entropies");
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
pub fn get_guessable(state: &str, forbidden: &str) -> String {
    let mut guessable = String::new();
    let excluded = [state, forbidden].concat();

    for letter in "abcdefghijklmnopqrstuvwxyz".chars() {
        if !excluded.contains(letter) {
            guessable.push(letter);
        }
    }

    guessable
}

/// A blazingly fast Rust command-line program to determine the best
/// letter to guess in a Hangman game.
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// The mystery word, with '?' representing unknown letters. Example: "h?llo"
    state: String,

    /// Any letters known not to be in the word
    #[arg(short, long, default_value_t = String::new())]
    incorrect: String,

    /// When enabled, list the remaining candidate words and exit.
    #[arg(short, long, default_value_t = false)]
    list: bool,
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // let prune_re = build_regex(&config.state, &config.incorrect)?;
    eprintln!("building wordlist");
    let mut word_set: HashSet<String> = ALL_WORDS.split('\n').map(String::from).collect();
    let data = GameData::new(&config.state, &config.incorrect);
    word_set = prune_wordlist(word_set, &data);

    if word_set.is_empty() {
        println!("There are no valid English words that match this game state.");
        return Ok(());
    }
    
    if word_set.len() == 1 {
        println!("The mystery word must be '{}'.", word_set.drain().next().expect("length of set must be 1"));
        return Ok(());
    }

    if config.list {
        for word in &word_set {
            println!("{word}");
        }
        
        eprintln!("{} possible words remaining", word_set.len());
        return Ok(());
    }
    
    let valid_letters = get_guessable(&config.state, &config.incorrect);
    eprintln!("Best letters to guess:");
    
    for letter_entropy in get_sorted_entropies(&valid_letters, &word_set) {
        println!("{}", letter_entropy);
    }
    
    eprintln!("{} possible words remaining", word_set.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    #[ignore = "regex no longer used"]
    fn test_build_regex() {
        let re = build_regex("A???e", "krbg").unwrap();
        assert_eq!(re, "^a[^aekrbg][^aekrbg][^aekrbg]e$".to_string());
    }

    #[test]
    #[ignore = "regex no longer used"]
    fn test_build_regex_with_invalid_characters() {
        let re = build_regex("*(*H)", "kjd");
        assert!(re.is_err());

        let re = build_regex("h?ll?", "(*&j)");
        assert!(re.is_err());
    }

    #[test]
    #[ignore = "regex no longer used"]
    fn test_build_regex_none_forbidden() {
        let re = build_regex("?????", "").unwrap();
        assert_eq!(re, "^.....$".to_string());
    }

    #[test]
    fn test_matches_constraints() {
        let state = "h????";
        let forbidden = "ay";
        let data = GameData::new(state, forbidden);

        assert!(matches_constraints("hello", &data));
        assert!(!matches_constraints("happy", &data));
        assert!(!matches_constraints("he", &data));
    }

    #[test]
    fn test_prune_wordlist() {
        // let re = build_regex("abc??", "ghi").unwrap();
        let data = GameData::new("abc??", "ghi");
        let words: HashSet<String> = ["abcde", "abcef", "abcfg", "abcgh", "abcbe", "abc"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let expected: HashSet<String> = ["abcde", "abcef"].iter().map(|s| s.to_string()).collect();
        let filtered = prune_wordlist(words, &data);

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
        let expected = HashSet::from_iter("bcdfghjklmpqruvwxyz".chars());

        assert_eq!(guessable, expected)
    }
    
}
