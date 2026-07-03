use std::collections::HashSet;

use hangstop::*;
use std::error::Error;

fn main() {
    println!("Hello, world!");
}

enum Guess {
    Letter(char),
    Word(String),
    NoSolution,
}

fn get_guess(state: &str, forbidden: &str) -> Result<Guess, Box<dyn Error>> {
    let re = build_regex(state, forbidden)?;
    let mut word_set: HashSet<String> = ALL_WORDS.split('\n').map(String::from).collect();
    word_set = prune_wordlist(re, word_set);

    if word_set.is_empty() {
        return Ok(Guess::NoSolution);
    }
    
    if word_set.len() == 1 {
        return Ok(Guess::Word(word_set.drain().next().expect("word_set must be length 1")));
    }

    let valid_letters = get_guessable(state, forbidden);
    
    Ok(Guess::Letter(get_sorted_entropies(&valid_letters, &word_set)[0].get_letter()))
}

struct Game<'a> {
    word: &'a str,
    state: String,
    forbidden: String
}

impl Game<'_> {
    fn new<'a>(word: &'a str) -> Game<'a> {
        Game { word, state: "?".repeat(word.len()), forbidden: String::new() }
    }

    fn update(&mut self, guess: char) {
        let mut found = false;

        for (i, letter) in self.word.chars().enumerate() {
            if letter == guess {
                found = true;
                self.state.remove(i);
                self.state.insert(i, letter);
            }
        }

        if !found {
            self.forbidden.push(guess);
        }
    }
}

