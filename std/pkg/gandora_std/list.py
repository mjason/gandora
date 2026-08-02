"""List-shape helpers; element-wise work lives in Enum (GEP-0010)."""

import gandora_std.enum


def _gan_truthy(value):
    return value is not None and value is not False


def first(xs):
    """The first element, or nil for an empty list."""
    return gandora_std.enum.at(xs, 0)


def last(xs):
    """The last element, or nil for an empty list."""
    return gandora_std.enum.at(xs, -1)


def flatten(xs):
    """Flattens nested lists to any depth.


      >>> flatten([1, [2, [3, 4]], 5])
      [1, 2, 3, 4, 5]
"""
    def _gan_fn0(x):
        if _gan_truthy(isinstance(x, list)):
            return flatten(x)
        else:
            return [x]
    return gandora_std.enum.flat_map(xs, _gan_fn0)


def wrap(value):
    """Wraps a value in a list: nil becomes [], lists pass through, anything else becomes [value]."""
    if _gan_truthy((value is None)):
        return []
    elif _gan_truthy(isinstance(value, list)):
        return value
    else:
        return [value]


def duplicate(value, n):
    """A list of `n` copies of `value`."""
    return ([(value)] * (n))


def insert_at(xs, index, value):
    """Inserts `value` at `index` (negative counts from the end, as in Elixir)."""
    return ((xs)[:(index)] + [(value)] + (xs)[(index):])


def delete_at(xs, index):
    """Removes the element at `index`; out-of-range leaves the list unchanged."""
    return ((xs)[:(index)] + (xs)[(index):][1:])
