#!/usr/bin/python


from typing import List


def read_words(filepath: str, n: int) -> List[str]:
    """
    Return an ordered list of words from the file,
    only including every `n` words.

    Newlines are kept at the end of each word
    """
    with open(filepath, "r", encoding="utf-8") as f:
        return [line for i, line in enumerate(f) if i % n == 0]

def write_words(filepath: str, words: List[str]) -> None:
    """Writes the words (assumed to end with newlines) to a file"""
    with open(filepath, "r", encoding="utf-8") as f:
        f.writelines(words)

if __name__ == "__main__":
    reduced = read_words("../public/wordlist.txt", 100)
    write_words("../public/reduced_wordlist.txt", reduced)
