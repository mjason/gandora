"""List-shape helpers; element-wise work lives in Enum (GEP-0010)."""

import builtins
import collections.abc
import typing
import gandora_std.enum

_T_a = typing.TypeVar("_T_a")


def first(xs: list[_T_a]) -> _T_a | None:
    """The first element, or nil for an empty list.

## Parameters

  - xs: The list.

    >>> first([1, 2, 3])
    1
"""
    return gandora_std.enum.at(xs, 0)


def last(xs: list[_T_a]) -> _T_a | None:
    """The last element, or nil for an empty list.

## Parameters

  - xs: The list.

    >>> last([1, 2, 3])
    3
"""
    return gandora_std.enum.at(xs, -1)


def flatten(xs: list) -> list:
    """Flattens nested lists to any depth.

## Parameters

  - xs: Nested lists collapse to one level.

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
    """Wraps a value in a list: nil becomes [], lists pass through, anything else becomes [value].

## Parameters

  - value: nil becomes [], lists pass through, others become [value].

    >>> wrap(1)
    [1]
"""
    if (value is None):
        return []
    elif isinstance(value, list):
        return value
    else:
        return [value]


def duplicate(value: _T_a, n: int) -> list[_T_a]:
    """A list of `n` copies of `value`.

## Parameters

  - value: The value to repeat.
  - n: How many copies.

    >>> duplicate("x", 3)
    ['x', 'x', 'x']
"""
    return ([(value)] * (n))


def insert_at(xs: list[_T_a], index: int, value: _T_a) -> list[_T_a]:
    """Inserts `value` at `index` (negative counts from the end, as in Elixir).

## Parameters

  - xs: The list.
  - index: Where the value lands.
  - value: The value to insert.

    >>> insert_at([1, 3], 1, 2)
    [1, 2, 3]
"""
    return ((xs)[:(index)] + [(value)] + (xs)[(index):])


def delete_at(xs: list[_T_a], index: int) -> list[_T_a]:
    """Removes the element at `index`; out-of-range leaves the list unchanged.

## Parameters

  - xs: The list.
  - index: The position to remove.

    >>> delete_at([1, 2, 3], 1)
    [1, 3]
"""
    return ((xs)[:(index)] + (xs)[(index):][1:])


def to_tuple(xs: list) -> tuple:
    """The list as a tuple.

## Parameters

  - xs: The list.

    >>> to_tuple([1, 2])
    (1, 2)
"""
    return builtins.tuple(xs)


def starts_with_p(xs: list[_T_a], prefix: list[_T_a]) -> bool:
    """Whether the list starts with `prefix`.

## Parameters

  - xs: The list.
  - prefix: The candidate prefix.

    >>> starts_with_p([1, 2, 3], [1, 2])
    True
"""
    return ((xs)[:len((prefix))] == (prefix))


def replace_at(xs: list[_T_a], index: int, value: _T_a) -> list[_T_a]:
    """Replaces the element at `index` (negative counts from the end).

## Parameters

  - xs: The list.
  - index: The position to replace.
  - value: The new value.

    >>> replace_at([1, 2, 3], 1, 9)
    [1, 9, 3]
"""
    return ((xs)[:(index)] + [(value)] + (xs)[(index):][1:])


def update_at(xs: list[_T_a], index: int, f: collections.abc.Callable) -> list[_T_a]:
    """Updates the element at `index` by `f`.

## Parameters

  - xs: The list.
  - index: The position to update.
  - f: Applied to the current value.

    >>> update_at([1, 2, 3], 1, lambda v: v * 10)
    [1, 20, 3]
"""
    return replace_at(xs, index, f(gandora_std.enum.at(xs, index)))
