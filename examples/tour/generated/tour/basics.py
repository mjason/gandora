"""Literals, operators, strings, and Elixir truthiness."""

import builtins


def _gan_truthy(value):
    return value is not None and value is not False

def _gan_div(a, b):
    q = a // b
    if q < 0 and q * b != a:
        q += 1
    return q


def _gan_rem(a, b):
    return a - _gan_div(a, b) * b


def literals() -> list:
    """One of every literal, in a list."""
    return [42, 1000000, 3.14, "ok", "quoted atom", "text", True, None, [1, 2, 3], ("pair", "tuple"), {"key": "value", "atom_key": 1}, [("name", "keyword"), ("list", True)]]


def arithmetic() -> tuple[int, float, int, int]:
    """Elixir arithmetic semantics: `/` is true division, `//` and `rem` truncate.

    >>> arithmetic()
    (13, 2.5, 2, -1)
"""
    a = 7 + (3 * 2)
    b = 10 / 4
    c = _gan_div(10, 4)
    d = _gan_rem(-7, 2)
    return (a, b, c, d)


def strings(name: str) -> tuple[str, str, str]:
    """Interpolation, Python methods on strings, and `<>` concatenation.

## Parameters

  - name: Spliced into the greeting.
"""
    greeting = f"Hello, {name}!"
    upper = greeting.upper()
    joined = "a b c" + " d"
    return (greeting, upper, joined)


def truthiness(x: object) -> str:
    """Only `false` and `nil` are falsy — 0 and empty containers are truthy.

## Parameters

  - x: Any value to test.
"""
    if _gan_truthy(x):
        return f"truthy: {repr(x)}"
    else:
        return f"falsy: {repr(x)}"


def demo() -> None:
    """Runs the chapter."""
    print(f"literals:   {repr(literals())}")
    print(f"arithmetic: {repr(arithmetic())}")
    _gan_fstr0 = repr(strings("gandora"))
    print(f"strings:    {_gan_fstr0}")
    print(truthiness(0))
    print(truthiness(None))
    total = builtins.sum(builtins.list(range(1, (10) + 1)))
    return print(f"sum of 1..10 = {total}")
