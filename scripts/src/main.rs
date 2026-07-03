use hangstop::*;

fn main() {
    println!("Hello, world!");
}

enum Guess {
    Letter(char),
    Word(String),
    NoSolution,
}
