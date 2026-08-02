"""List-shape helpers; element-wise work lives in Enum (GEP-0010)."""

import builtins
import collections.abc
import gandora_std.enum


def first(xs: list) -> object:
    """The first element, or nil for an empty list."""
    return gandora_std.enum.at(xs, 0)


def last(xs: list) -> object:
    """The last element, or nil for an empty list."""
    return gandora_std.enum.at(xs, -1)


def flatten(xs: list) -> list:
    """Flattens nested lists to any depth.

    >>> flatten([1, [2, [3, 4]], 5])
    [1, 2, 3, 4, 5]
"""
    def _gan_fn0(x):
        if isinstance(x, list):
            return flatten(x)
        else:
            return [x]
    return gandora_std.enum.flat_map(xs, _gan_fn0)


def wrap(value: object) -> list:
    """Wraps a value in a list: nil becomes [], lists pass through, anything else becomes [value]."""
    if (value is None):
        return []
    elif isinstance(value, list):
        return value
    else:
        return [value]


def duplicate(value: object, n: int) -> list:
    """A list of `n` copies of `value`."""
    return ([(value)] * (n))


def insert_at(xs: list, index: int, value: object) -> list:
    """Inserts `value` at `index` (negative counts from the end, as in Elixir)."""
    return ((xs)[:(index)] + [(value)] + (xs)[(index):])


def delete_at(xs: list, index: int) -> list:
    """Removes the element at `index`; out-of-range leaves the list unchanged."""
    return ((xs)[:(index)] + (xs)[(index):][1:])


def to_tuple(xs: list) -> tuple:
    """The list as a tuple."""
    return builtins.tuple(xs)


def starts_with_p(xs: list, prefix: list) -> bool:
    """Whether the list starts with `prefix`."""
    return ((xs)[:len((prefix))] == (prefix))


def replace_at(xs: list, index: int, value: object) -> list:
    """Replaces the element at `index` (negative counts from the end)."""
    return ((xs)[:(index)] + [(value)] + (xs)[(index):][1:])


def update_at(xs: list, index: int, f: collections.abc.Callable) -> list:
    """Updates the element at `index` by `f`.

    >>> update_at([1, 2, 3], 1, lambda v: v * 10)
    [1, 20, 3]
"""
    return replace_at(xs, index, f(gandora_std.enum.at(xs, index)))
