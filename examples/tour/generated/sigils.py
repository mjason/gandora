"""The sigil family: ~w word lists, ~s strings without escaping, ~r
regexes as module attributes, and ~python for expressions that only
Python can say.
"""

import re

word_re = re.compile("\\w+")


def words(text: str) -> list[str]:
    """The words of a text — the ~r sigil compiled once, reused everywhere.

## Parameters

  - text: Any text; \\w matches Unicode word characters.

    >>> words("hello, 世界 world!")
    ['hello', '世界', 'world']
"""
    return word_re.findall(text)


def main() -> None:
    """Runs the chapter."""
    print(repr(["gandora", "elixir", "python"]))
    print("no need to escape \"quotes\" here")
    print(repr(words("hello, 世界 world!")))
    total = (sum(i * i for i in range(10)))
    print(f"sum of squares: {total}")
    evens = (lambda xs: [x for x in xs if x % 2 == 0])([1, 2, 3, 4, 5, 6])
    return print(repr(evens))


if __name__ == "__main__":
    main()
