"""Data-first string functions over Python str (GEP-0010)."""

import builtins
import re
import gandora_std.enum


def upcase(s: str) -> str:
    """Uppercases the string.

## Parameters

  - s: The string.
"""
    return s.upper()


def downcase(s: str) -> str:
    """Lowercases the string.

## Parameters

  - s: The string.
"""
    return s.lower()


def capitalize(s: str) -> str:
    """Uppercases the first character, lowercases the rest.

## Parameters

  - s: The string.
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
"""
    return s.split(sep)


def trim(s: str) -> str:
    """Removes leading and trailing whitespace.

## Parameters

  - s: The string.
"""
    return s.strip()


def replace(s: str, pattern: str, replacement: str) -> str:
    """Replaces every occurrence of `pattern` with `replacement`.

## Parameters

  - s: The string.
  - pattern: The substring to find.
  - replacement: Substituted for every occurrence.
"""
    return s.replace(pattern, replacement)


def contains_p(s: str, part: str) -> bool:
    """Whether the string contains `part`.

## Parameters

  - s: The string.
  - part: The substring to look for.
"""
    return ((part) in (s))


def starts_with_p(s: str, prefix: str) -> bool:
    """Whether the string starts with `prefix`.

## Parameters

  - s: The string.
  - prefix: The candidate prefix.
"""
    return s.startswith(prefix)


def ends_with_p(s: str, suffix: str) -> bool:
    """Whether the string ends with `suffix`.

## Parameters

  - s: The string.
  - suffix: The candidate suffix.
"""
    return s.endswith(suffix)


def length(s: str) -> int:
    """The number of characters (Unicode code points, Python `len`).

## Parameters

  - s: The string.
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
"""
    return s.rjust(width)


def pad_trailing(s: str, width: int) -> str:
    """Pads on the right with spaces to `width`.

## Parameters

  - s: The string.
  - width: The minimum total width.
"""
    return s.ljust(width)


def to_integer(s: str) -> int:
    """Parses an integer; raises Python ValueError on bad input.

## Parameters

  - s: The decimal digits to parse.
"""
    return builtins.int(s)


def to_float(s: str) -> float:
    """Parses a float; raises Python ValueError on bad input.

## Parameters

  - s: The number literal to parse.
"""
    return builtins.float(s)


def at(s: str, index: int) -> str | None:
    """The character at `index` (negative counts from the end), or nil.

## Parameters

  - s: The string.
  - index: Zero-based; negative counts from the end.
"""
    if (index < builtins.len(s)) and (index >= -(builtins.len(s))):
        return ((s)[(index)])
    else:
        return None


def reverse(s: str) -> str:
    """The string reversed.

## Parameters

  - s: The string.
"""
    return ((s)[::-1])


def duplicate(s: str, n: int) -> str:
    """The string repeated `n` times.

## Parameters

  - s: The string to repeat.
  - n: How many times.
"""
    return ((s) * (n))


def trim_leading(s: str) -> str:
    """Removes leading whitespace.

## Parameters

  - s: The string.
"""
    return s.lstrip()


def trim_trailing(s: str) -> str:
    """Removes trailing whitespace.

## Parameters

  - s: The string.
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

    >>> match_p("gandora-2026", re.compile("\\d+"))
    True
"""
    return not ((regex.search(s) is None))
