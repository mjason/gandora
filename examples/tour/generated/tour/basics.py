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


def literals():
    return [42, 1000000, 3.14, "ok", "quoted atom", "text", True, None, [1, 2, 3], ("pair", "tuple"), {"key": "value", "atom_key": 1}, [("name", "keyword"), ("list", True)]]


def arithmetic():
    a = 7 + (3 * 2)
    b = 10 / 4
    c = _gan_div(10, 4)
    d = _gan_rem(-7, 2)
    return (a, b, c, d)


def strings(name):
    greeting = f"Hello, {name}!"
    upper = greeting.upper()
    joined = "a b c" + " d"
    return (greeting, upper, joined)


def truthiness(x):
    if _gan_truthy(x):
        return f"truthy: {repr(x)}"
    else:
        return f"falsy: {repr(x)}"


def demo():
    print(f"literals:   {repr(literals())}")
    print(f"arithmetic: {repr(arithmetic())}")
    print(f"strings:    {repr(strings("gandora"))}")
    print(truthiness(0))
    print(truthiness(None))
    total = builtins.sum(builtins.list(range(1, (10) + 1)))
    return print(f"sum of 1..10 = {total}")
