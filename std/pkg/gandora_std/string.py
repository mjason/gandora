"""Data-first string functions over Python str (GEP-0010)."""

import builtins
import re
import gandora_std.enum


def _gan_truthy(value):
    return value is not None and value is not False


def upcase(s):
    """Uppercases the string."""
    return s.upper()


def downcase(s):
    """Lowercases the string."""
    return s.lower()


def capitalize(s):
    """Uppercases the first character, lowercases the rest."""
    return s.capitalize()


def split(s):
    """Splits on whitespace runs, dropping empty parts (Elixir semantics).


      >>> split("  a  b c ")
      ['a', 'b', 'c']
"""
    return s.split()


def split_on(s, sep):
    """Splits on the separator `sep`."""
    return s.split(sep)


def trim(s):
    """Removes leading and trailing whitespace."""
    return s.strip()


def replace(s, pattern, replacement):
    """Replaces every occurrence of `pattern` with `replacement`."""
    return s.replace(pattern, replacement)


def contains_p(s, part):
    """Whether the string contains `part`."""
    return ((part) in (s))


def starts_with_p(s, prefix):
    """Whether the string starts with `prefix`."""
    return s.startswith(prefix)


def ends_with_p(s, suffix):
    """Whether the string ends with `suffix`."""
    return s.endswith(suffix)


def length(s):
    """The number of characters (Unicode code points, Python `len`)."""
    return builtins.len(s)


def slice(s, start, len):
    """The substring of `len` characters starting at `start` (negative start counts from the end).


      >>> slice("gandora", 3, 4)
      'dora'
"""
    return ((s)[(start):] [:(len)])


def pad_leading(s, width):
    """Pads on the left with spaces to `width`."""
    return s.rjust(width)


def pad_trailing(s, width):
    """Pads on the right with spaces to `width`."""
    return s.ljust(width)


def to_integer(s):
    """Parses an integer; raises Python ValueError on bad input."""
    return builtins.int(s)


def to_float(s):
    """Parses a float; raises Python ValueError on bad input."""
    return builtins.float(s)


def at(s, index):
    """The character at `index` (negative counts from the end), or nil."""
    if (index < builtins.len(s)) and (index >= -(builtins.len(s))):
        return ((s)[(index)])
    else:
        return None


def reverse(s):
    """The string reversed."""
    return ((s)[::-1])


def duplicate(s, n):
    """The string repeated `n` times."""
    return ((s) * (n))


def trim_leading(s):
    """Removes leading whitespace."""
    return s.lstrip()


def trim_trailing(s):
    """Removes trailing whitespace."""
    return s.rstrip()


def codepoints(s):
    """The characters as a list of one-character strings.


      >>> gandora_std.enum.take(codepoints("héllo"), 2)
      ['h', 'é']
"""
    return builtins.list(s)


def match_p(s, regex):
    """Whether the compiled regex (`~r/.../`) matches anywhere in the string.


      >>> match_p("gandora-2026", re.compile("\\d+"))
      True
"""
    return not (_gan_truthy((regex.search(s) is None)))
