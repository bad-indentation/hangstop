use std::collections::{HashSet, HashMap};
use std::fs::read_to_string;

use hangstop::*;

use regex::Regex;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>>{
    let small_wordlist = read_to_string("../public/reduced_wordlist.txt")?;
    let small_wordlist: HashSet<String> = small_wordlist.split('\n').map(String::from).collect();

    let mut results = get_data(small_wordlist)?;
    
    let mut numbered: Vec<(usize, GuessData)> = results.drain().collect();

    numbered.sort_by_key(|pair| pair.0);

    println!("length, avg. guesses, avg. incorrect");
    for (length, data) in numbered {
        data.print_csv_line(length);
    }

    Ok(())
}

/// Informtion returned by the get_guess function, including
/// the guess and the filtered wordlist
enum Guess {
    Letter(char, HashSet<String>),
    Word(String),
    NoSolution,
}

fn get_guess(state: &str, forbidden: &str, mut word_set: HashSet<String>) -> Result<Guess, Box<dyn Error>> {
    let re = build_regex(state, forbidden)?;
    let re = Regex::new(&re).expect("regex should be valid.");
    word_set = HashSet::from_iter(word_set.iter().filter(|word| re.is_match(word)).map(String::from));

    if word_set.is_empty() {
        return Ok(Guess::NoSolution);
    }
    
    if word_set.len() == 1 {
        let word = word_set.drain().next().expect("word_set must be length 1");
        return Ok(Guess::Word(word));
    }

    let valid_letters = get_guessable(state, forbidden);
    
    Ok(Guess::Letter(get_sorted_entropies(&valid_letters, &word_set)[0].get_letter(), word_set))
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

/// For any given word length, represents the data needed to compute the
/// average number of guesses and the average number of incorrect guesses 
/// for that word
struct GuessData {
    total_guesses: usize,
    total_incorrect_guesses: usize,
    total_games: usize,
}

impl GuessData {
    fn get_average_guesses(&self) -> f32 {
        self.total_guesses as f32 / self.total_games as f32
    }

    fn get_average_incorrect(&self) -> f32 {
        self.total_incorrect_guesses as f32 / self.total_games as f32
    }

    fn print_csv_line(&self, word_length: usize) {
        println!("{}, {}, {}", word_length, self.get_average_guesses(), self.get_average_incorrect());
    }
}

/// Iterates ove the list of words and returns a hashmap mapping a possible 
/// word length to the total number of games with that length, the total number
/// of guesses, and the total number of incorrect guesses
fn get_data(test_list: HashSet<String>) -> Result<HashMap<usize, GuessData>, Box<dyn Error>> {
    let mut data = HashMap::new();

    let original_word_set: HashSet<String> = ALL_WORDS.split('\n').map(String::from).collect();
    for word in test_list {
        eprintln!("{word}");

        let mut word_set = original_word_set.clone();
        let mut game = Game::new(&word);
        let count = data.entry(word.len()).or_insert(GuessData { total_guesses: 0, total_incorrect_guesses: 0, total_games: 0 });

        count.total_games += 1;

        loop {
            let guess = get_guess(&game.state, &game.forbidden, word_set)?;
            eprintln!("{}, {}", &game.state, &game.forbidden);
            match guess {
                Guess::Letter(letter, set) => {
                    count.total_guesses += 1;

                    if !word.contains(letter) {
                        count.total_incorrect_guesses += 1;
                    }
                    game.update(letter);

                    word_set = set;
                }

                Guess::Word(guessed_word) => {
                    if guessed_word == word {
                        break;
                    }

                    eprintln!("guesses {guessed_word} for secret word {word} but it was not the answer.");
                    return Err("guessed word not the answer.".into());
                }

                Guess::NoSolution => {
                    eprintln!("when guessing {word}:");
                    return Err("no solution occured, but test_list SHOULD be a subset of the original wordlist".into());

                }
            }

        }
    }

    Ok(data)
}
