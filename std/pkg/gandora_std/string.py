"""Data-first string functions over Python str (GEP-0010).

Number formatting is Python's own format mini-language, called as a
method on a string literal: `"{:.2f}".format(3.14159)` gives '3.14',
`"{:>8}".format(x)` pads — no wrapper needed.
"""

import builtins
import re
import gandora_std.enum


def upcase(s: str) -> str:
    """Uppercases the string.

## Parameters

  - s: The string.

    >>> upcase("abc")
    'ABC'
"""
    return s.upper()


def downcase(s: str) -> str:
    """Lowercases the string.

## Parameters

  - s: The string.

    >>> downcase("AbC")
    'abc'
"""
    return s.lower()


def capitalize(s: str) -> str:
    """Uppercases the first character, lowercases the rest.

## Parameters

  - s: The string.

    >>> capitalize("heLLo")
    'Hello'
"""
    return s.capitalize()


def split(s: str) -> list[str]:
    """Splits on whitespace runs, dropping empty parts (Elixir semantics).

## Parameters

  - s: Split on runs of whitespace.

    >>> split("  a  b c ")
    ['a', 'b', 'c']
"""
    return s.split()


def split_on(s: str, sep: str) -> list[str]:
    """Splits on the separator `sep`.

## Parameters

  - s: The string.
  - sep: The separator to split on.

    >>> split_on("a,b,c", ",")
    ['a', 'b', 'c']
"""
    return s.split(sep)


def trim(s: str) -> str:
    """Removes leading and trailing whitespace.

## Parameters

  - s: The string.

    >>> trim("  x  ")
    'x'
"""
    return s.strip()


def replace(s: str, pattern: str, replacement: str) -> str:
    """Replaces every occurrence of `pattern` with `replacement`.

## Parameters

  - s: The string.
  - pattern: The substring to find.
  - replacement: Substituted for every occurrence.

    >>> replace("a-b-a", "a", "x")
    'x-b-x'
"""
    return s.replace(pattern, replacement)


def contains_p(s: str, part: str) -> bool:
    """Whether the string contains `part`.

## Parameters

  - s: The string.
  - part: The substring to look for.

    >>> contains_p("hello", "ell")
    True
"""
    return ((part) in (s))


def starts_with_p(s: str, prefix: str) -> bool:
    """Whether the string starts with `prefix`.

## Parameters

  - s: The string.
  - prefix: The candidate prefix.

    >>> starts_with_p("file.gan", "file")
    True
"""
    return s.startswith(prefix)


def ends_with_p(s: str, suffix: str) -> bool:
    """Whether the string ends with `suffix`.

## Parameters

  - s: The string.
  - suffix: The candidate suffix.

    >>> ends_with_p("file.gan", ".gan")
    True
"""
    return s.endswith(suffix)


def length(s: str) -> int:
    """The number of characters (Unicode code points, Python `len`).

## Parameters

  - s: The string.

    >>> length("héllo")
    5
"""
    return builtins.len(s)


def slice(s: str, start: int, len: int) -> str:
    """The substring of `len` characters starting at `start` (negative start counts from the end).

## Parameters

  - s: The string.
  - start: Zero-based start position.
  - len: Maximum number of characters.

    >>> slice("gandora", 3, 4)
    'dora'
"""
    return ((s)[(start):] [:(len)])


def pad_leading(s: str, width: int) -> str:
    """Pads on the left with spaces to `width`.

## Parameters

  - s: The string.
  - width: The minimum total width.

    >>> pad_leading("5", 3)
    '  5'
"""
    return s.rjust(width)


def pad_trailing(s: str, width: int) -> str:
    """Pads on the right with spaces to `width`.

## Parameters

  - s: The string.
  - width: The minimum total width.

    >>> pad_trailing("5", 3)
    '5  '
"""
    return s.ljust(width)


def to_integer(s: str) -> int:
    """Parses an integer; raises Python ValueError on bad input.

## Parameters

  - s: The decimal digits to parse.

    >>> to_integer("42")
    42
"""
    return builtins.int(s)


def to_float(s: str) -> float:
    """Parses a float; raises Python ValueError on bad input.

## Parameters

  - s: The number literal to parse.

    >>> to_float("2.5")
    2.5
"""
    return builtins.float(s)


def at(s: str, index: int) -> str | None:
    """The character at `index` (negative counts from the end), or nil.

## Parameters

  - s: The string.
  - index: Zero-based; negative counts from the end.

    >>> at("abc", 1)
    'b'
"""
    if (index < builtins.len(s)) and (index >= -(builtins.len(s))):
        return ((s)[(index)])
    else:
        return None


def first(s: str) -> str | None:
    """The first character, or nil for the empty string.

## Parameters

  - s: The string.

    >>> first("ant")
    'a'
    >>> first("")
"""
    return at(s, 0)


def last(s: str) -> str | None:
    """The last character, or nil for the empty string.

## Parameters

  - s: The string.

    >>> last("ant")
    't'
"""
    return at(s, -1)


def reverse(s: str) -> str:
    """The string reversed.

## Parameters

  - s: The string.

    >>> reverse("abc")
    'cba'
"""
    return ((s)[::-1])


def duplicate(s: str, n: int) -> str:
    """The string repeated `n` times.

## Parameters

  - s: The string to repeat.
  - n: How many times.

    >>> duplicate("ab", 2)
    'abab'
"""
    return ((s) * (n))


def trim_leading(s: str) -> str:
    """Removes leading whitespace.

## Parameters

  - s: The string.

    >>> trim_leading("  x")
    'x'
"""
    return s.lstrip()


def trim_trailing(s: str) -> str:
    """Removes trailing whitespace.

## Parameters

  - s: The string.

    >>> trim_trailing("x  ")
    'x'
"""
    return s.rstrip()


def codepoints(s: str) -> list[str]:
    """The characters as a list of one-character strings.

## Parameters

  - s: The string.

    >>> gandora_std.enum.take(codepoints("héllo"), 2)
    ['h', 'é']
"""
    return builtins.list(s)


def match_p(s: str, regex: re.Pattern) -> bool:
    """Whether the compiled regex (`~r/.../`) matches anywhere in the string.

## Parameters

  - s: The string to test.
  - regex: A compiled pattern, e.g. from ~r//.

    >>> match_p("gandora-2026", re.compile("\\\\d+"))
    True
"""
    return not ((regex.search(s) is None))
