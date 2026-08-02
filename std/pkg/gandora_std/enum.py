"""Eager, data-first collection functions (GEP-0010). Every subject comes first, so everything pipes."""

import builtins
import functools
import gandora_std.enum
import gandora_std.string


def _gan_div(a, b):
    q = a // b
    if q < 0 and q * b != a:
        q += 1
    return q


def _gan_rem(a, b):
    return a - _gan_div(a, b) * b


def map(xs, f):
    """Applies `f` to every element.


      >>> gandora_std.enum.map([1, 2, 3], lambda x: x * 10)
      [10, 20, 30]
"""
    return builtins.list(builtins.map(f, xs))


def filter(xs, f):
    """Keeps elements for which `f` is truthy.


      >>> gandora_std.enum.filter([1, 2, 3, 4], lambda x: _gan_rem(x, 2) == 0)
      [2, 4]
"""
    return ([x for x in (xs) if (f)(x)])


def reject(xs, f):
    """Drops elements for which `f` is truthy."""
    return ([x for x in (xs) if not (f)(x)])


def reduce(xs, acc, f):
    """Folds left with Elixir argument order: `f` receives (element, acc).


      >>> gandora_std.enum.reduce([1, 2, 3, 4], 0, lambda x, acc: acc + x)
      10
"""
    return functools.reduce(lambda a, x: f(x, a), xs, acc)


def sum(xs):
    """Sum of the elements."""
    return builtins.sum(xs)


def count(xs):
    """Number of elements."""
    return builtins.len(xs)


def sort(xs):
    """Ascending sort."""
    return builtins.sorted(xs)


def sort_by(xs, f):
    """Sorts by the key computed by `f` (Python `key=`, not a comparator).


      >>> gandora_std.enum.sort_by(["ccc", "a", "bb"], lambda s: gandora_std.string.length(s))
      ['a', 'bb', 'ccc']
"""
    return builtins.sorted(xs, key=f)


def reverse(xs):
    """Elements in reverse order."""
    return (list(reversed((xs))))


def join(xs, sep):
    """Joins elements (converted by `to_string`) with `sep`.


      >>> gandora_std.enum.join([1, 2, 3], "-")
      '1-2-3'
"""
    return sep.join(([str(x) for x in (xs)]))


def at(xs, index):
    """The element at `index` (negative counts from the end), or nil."""
    if (index < builtins.len(xs)) and (index >= -(builtins.len(xs))):
        return ((xs)[(index)])
    else:
        return None


def take(xs, n):
    """The first `n` elements."""
    return ((xs)[:(n)])


def drop(xs, n):
    """The elements after the first `n`."""
    return ((xs)[(n):])


def zip(xs, ys):
    """Pairs up two lists into `{a, b}` tuples, stopping at the shorter."""
    return builtins.list(builtins.zip(xs, ys))


def with_index(xs):
    """Each element paired with its index: `{element, index}`.


      >>> gandora_std.enum.with_index(["a", "b"])
      [('a', 0), ('b', 1)]
"""
    return ([(x, i) for i, x in enumerate((xs))])


def member_p(xs, x):
    """Whether `x` is an element."""
    return ((x) in (xs))


def all_p(xs, f):
    """Whether `f` is truthy for every element."""
    return (all((f)(x) for x in (xs)))


def any_p(xs, f):
    """Whether `f` is truthy for any element."""
    return (any((f)(x) for x in (xs)))


def empty_p(xs):
    """Whether the collection has no elements."""
    return builtins.len(xs) == 0


def uniq(xs):
    """Removes duplicates, keeping first occurrences in order."""
    return (list(dict.fromkeys((xs))))


def flat_map(xs, f):
    """Maps `f` (which returns a list) and concatenates the results.


      >>> gandora_std.enum.flat_map([1, 2], lambda x: [x, x * 10])
      [1, 10, 2, 20]
"""
    return ([y for x in (xs) for y in (f)(x)])


def each(xs, f):
    """Runs `f` on each element for its side effects; returns :ok."""
    ([(f)(x) for x in (xs)])
    return "ok"


def min(xs):
    """The smallest element."""
    return builtins.min(xs)


def max(xs):
    """The largest element."""
    return builtins.max(xs)
