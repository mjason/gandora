"""Eager, data-first collection functions (GEP-0010). Every subject comes first, so everything pipes."""

import builtins
import collections
import collections.abc
import functools
import itertools
import math
import gandora_std.string


def _gan_div(a, b):
    q = a // b
    if q < 0 and q * b != a:
        q += 1
    return q


def _gan_rem(a, b):
    return a - _gan_div(a, b) * b


def map(xs: list, f: collections.abc.Callable) -> list:
    """Applies `f` to every element.

    >>> map([1, 2, 3], lambda x: x * 10)
    [10, 20, 30]
"""
    return builtins.list(builtins.map(f, xs))


def filter(xs: list, f: collections.abc.Callable) -> list:
    """Keeps elements for which `f` is truthy.

    >>> filter([1, 2, 3, 4], lambda x: _gan_rem(x, 2) == 0)
    [2, 4]
"""
    return ([x for x in (xs) if (f)(x)])


def reject(xs: list, f: collections.abc.Callable) -> list:
    """Drops elements for which `f` is truthy."""
    return ([x for x in (xs) if not (f)(x)])


def reduce(xs: list, acc: object, f: collections.abc.Callable) -> object:
    """Folds left with Elixir argument order: `f` receives (element, acc).

    >>> reduce([1, 2, 3, 4], 0, lambda x, acc: acc + x)
    10
"""
    return functools.reduce(lambda a, x: f(x, a), xs, acc)


def sum(xs: list) -> int | float:
    """Sum of the elements."""
    return builtins.sum(xs)


def count(xs: list) -> int:
    """Number of elements."""
    return builtins.len(xs)


def sort(xs: list) -> list:
    """Ascending sort."""
    return builtins.sorted(xs)


def sort_by(xs: list, f: collections.abc.Callable) -> list:
    """Sorts by the key computed by `f` (Python `key=`, not a comparator).

    >>> sort_by(["ccc", "a", "bb"], lambda s: gandora_std.string.length(s))
    ['a', 'bb', 'ccc']
"""
    return builtins.sorted(xs, key=f)


def reverse(xs: list) -> list:
    """Elements in reverse order."""
    return (list(reversed((xs))))


def join(xs: list, sep: str) -> str:
    """Joins elements (converted by `to_string`) with `sep`.

    >>> join([1, 2, 3], "-")
    '1-2-3'
"""
    return sep.join(([str(x) for x in (xs)]))


def at(xs: list, index: int) -> object:
    """The element at `index` (negative counts from the end), or nil."""
    if (index < builtins.len(xs)) and (index >= -(builtins.len(xs))):
        return ((xs)[(index)])
    else:
        return None


def take(xs: list, n: int) -> list:
    """The first `n` elements."""
    return ((xs)[:(n)])


def drop(xs: list, n: int) -> list:
    """The elements after the first `n`."""
    return ((xs)[(n):])


def zip(xs: list, ys: list) -> list[tuple[object, object]]:
    """Pairs up two lists into `{a, b}` tuples, stopping at the shorter."""
    return builtins.list(builtins.zip(xs, ys))


def with_index(xs: list) -> list[tuple[object, int]]:
    """Each element paired with its index: `{element, index}`.

    >>> with_index(["a", "b"])
    [('a', 0), ('b', 1)]
"""
    return ([(x, i) for i, x in enumerate((xs))])


def member_p(xs: list, x: object) -> bool:
    """Whether `x` is an element."""
    return ((x) in (xs))


def all_p(xs: list, f: collections.abc.Callable) -> bool:
    """Whether `f` is truthy for every element."""
    return (all((f)(x) for x in (xs)))


def any_p(xs: list, f: collections.abc.Callable) -> bool:
    """Whether `f` is truthy for any element."""
    return (any((f)(x) for x in (xs)))


def empty_p(xs: list) -> bool:
    """Whether the collection has no elements."""
    return builtins.len(xs) == 0


def uniq(xs: list) -> list:
    """Removes duplicates, keeping first occurrences in order."""
    return (list(dict.fromkeys((xs))))


def flat_map(xs: list, f: collections.abc.Callable) -> list:
    """Maps `f` (which returns a list) and concatenates the results.

    >>> flat_map([1, 2], lambda x: [x, x * 10])
    [1, 10, 2, 20]
"""
    return ([y for x in (xs) for y in (f)(x)])


def each(xs: list, f: collections.abc.Callable) -> str:
    """Runs `f` on each element for its side effects; returns :ok."""
    ([(f)(x) for x in (xs)])
    return "ok"


def min(xs: list) -> object:
    """The smallest element."""
    return builtins.min(xs)


def max(xs: list) -> object:
    """The largest element."""
    return builtins.max(xs)


def find(xs: list, f: collections.abc.Callable) -> object:
    """The first element for which `f` is truthy, or nil.

    >>> find([1, 2, 3, 4], lambda x: x > 2)
    3
"""
    return (next((x for x in (xs) if (f)(x)), None))


def find_index(xs: list, f: collections.abc.Callable) -> int | None:
    """The index of the first element for which `f` is truthy, or nil."""
    return (next((i for i, x in enumerate((xs)) if (f)(x)), None))


def frequencies(xs: list) -> dict[object, int]:
    """A map from each distinct element to its occurrence count.

    >>> frequencies(["a", "b", "a"])
    {'a': 2, 'b': 1}
"""
    return builtins.dict(collections.Counter(xs))


def group_by(xs: list, f: collections.abc.Callable) -> dict:
    """Groups elements by the key computed by `f`, preserving order.

    >>> group_by([1, 2, 3, 4, 5], lambda x: _gan_rem(x, 2))
    {1: [1, 3, 5], 0: [2, 4]}
"""
    return ({k: [x for x in (xs) if (f)(x) == k] for k in dict.fromkeys((f)(x) for x in (xs))})


def max_by(xs: list, f: collections.abc.Callable) -> object:
    """The element maximizing the key computed by `f`; raises on empty."""
    return builtins.max(xs, key=f)


def min_by(xs: list, f: collections.abc.Callable) -> object:
    """The element minimizing the key computed by `f`; raises on empty."""
    return builtins.min(xs, key=f)


def product(xs: list) -> int | float:
    """The product of the elements (1 for an empty collection)."""
    return math.prod(xs)


def take_while(xs: list, f: collections.abc.Callable) -> list:
    """Leading elements while `f` stays truthy."""
    return builtins.list(itertools.takewhile(f, xs))


def drop_while(xs: list, f: collections.abc.Callable) -> list:
    """Drops leading elements while `f` stays truthy, keeps the rest."""
    return builtins.list(itertools.dropwhile(f, xs))


def chunk_every(xs: list, n: int) -> list[list]:
    """Splits into chunks of `n` elements; the last chunk may be shorter.

    >>> chunk_every([1, 2, 3, 4, 5], 2)
    [[1, 2], [3, 4], [5]]
"""
    return ([(xs)[i:i + (n)] for i in range(0, len((xs)), (n))])


def concat(xss: list[list]) -> list:
    """Concatenates a list of lists (one level)."""
    return ([y for x in (xss) for y in x])


def intersperse(xs: list, sep: object) -> list:
    """Puts `sep` between every two elements.

    >>> intersperse([1, 2, 3], "x")
    [1, 'x', 2, 'x', 3]
"""
    return ([v for x in (xs) for v in (x, (sep))][:-1])


def slice(xs: list, start: int, count: int) -> list:
    """`count` elements starting at `start` (negative start counts from the end)."""
    return ((xs)[(start):] [:(count)])


def dedup(xs: list) -> list:
    """Collapses consecutive duplicate elements.

    >>> dedup([1, 1, 2, 2, 2, 1])
    [1, 2, 1]
"""
    return ([x for i, x in enumerate((xs)) if i == 0 or x != (xs)[i - 1]])
