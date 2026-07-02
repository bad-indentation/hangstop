#!/usr/bin/python


from typing import List
from pathlib import Path


def read_words(filepath: str | Path, n: int) -> List[str]:
    """
    Return an ordered list of words from the file,
    only including every `n` words.

    Newlines are kept at the end of each word
    """
    with open(filepath, "r", encoding="utf-8") as f:
        return [line for i, line in enumerate(f) if i % n == 0]

def write_words(filepath: str | Path, words: List[str]) -> None:
    """Writes the words (assumed to end with newlines) to a file"""
    with open(filepath, "w", encoding="utf-8") as f:
        f.writelines(words)

if __name__ == "__main__":
    wordlist_path = Path("../public/wordlist.txt")
    reduced = read_words(wordlist_path.resolve(), 100)

    output_path = Path("../public/reduced_wordlist.txt")
    write_words(output_path.resolve(), reduced)
