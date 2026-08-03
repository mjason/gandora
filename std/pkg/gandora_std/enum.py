"""Eager, data-first collection functions (GEP-0010). Every subject comes first, so everything pipes."""

import builtins
import collections
import collections.abc
import functools
import itertools
import math
import typing
import gandora_std.string


def _gan_div(a, b):
    q = a // b
    if q < 0 and q * b != a:
        q += 1
    return q


def _gan_rem(a, b):
    return a - _gan_div(a, b) * b

_T_a = typing.TypeVar("_T_a")
_T_b = typing.TypeVar("_T_b")


def map(xs: list[_T_a], f: collections.abc.Callable) -> list[_T_b]:
    """Applies `f` to every element.

## Parameters

  - xs: The source list.
  - f: Applied to each element; the results form the new list.

    >>> map([1, 2, 3], lambda x: x * 10)
    [10, 20, 30]
"""
    return builtins.list(builtins.map(f, xs))


def filter(xs: list[_T_a], f: collections.abc.Callable) -> list[_T_a]:
    """Keeps elements for which `f` is truthy.

## Parameters

  - xs: The source list.
  - f: Keeps the elements it returns truthy for.

    >>> filter([1, 2, 3, 4], lambda x: _gan_rem(x, 2) == 0)
    [2, 4]
"""
    return ([x for x in (xs) if (f)(x)])


def reject(xs: list[_T_a], f: collections.abc.Callable) -> list[_T_a]:
    """Drops elements for which `f` is truthy.

## Parameters

  - xs: The source list.
  - f: Drops the elements it returns truthy for.

    >>> reject([1, 2, 3, 4], lambda x: _gan_rem(x, 2) == 0)
    [1, 3]
"""
    return ([x for x in (xs) if not (f)(x)])


def reduce(xs: list[_T_a], acc: _T_b, f: collections.abc.Callable) -> _T_b:
    """Folds left with Elixir argument order: `f` receives (element, acc).

## Parameters

  - xs: The list to fold.
  - acc: The starting accumulator.
  - f: Receives an element and the accumulator, returns the next accumulator.

    >>> reduce([1, 2, 3, 4], 0, lambda x, acc: acc + x)
    10
"""
    return functools.reduce(lambda a, x, *, f=f: f(x, a), xs, acc)


def sum(xs: list) -> int | float:
    """Sum of the elements.

## Parameters

  - xs: The numbers to add.

    >>> sum([1, 2, 3])
    6
"""
    return builtins.sum(xs)


def count(xs: list) -> int:
    """Number of elements.

## Parameters

  - xs: The list to measure.

    >>> count([10, 20, 30])
    3
"""
    return builtins.len(xs)


def sort(xs: list) -> list:
    """Ascending sort.

## Parameters

  - xs: The list to order ascending.

    >>> sort([3, 1, 2])
    [1, 2, 3]
"""
    return builtins.sorted(xs)


def sort_by(xs: list[_T_a], f: collections.abc.Callable) -> list[_T_a]:
    """Sorts by the key computed by `f` (Python `key=`, not a comparator).

## Parameters

  - xs: The list to order.
  - f: Maps each element to its sort key.

    >>> sort_by(["ccc", "a", "bb"], lambda s: gandora_std.string.length(s))
    ['a', 'bb', 'ccc']
"""
    return builtins.sorted(xs, key=f)


def reverse(xs: list[_T_a]) -> list[_T_a]:
    """Elements in reverse order.

## Parameters

  - xs: The list to reverse.

    >>> reverse([1, 2, 3])
    [3, 2, 1]
"""
    return (list(reversed((xs))))


def join(xs: list, sep: str) -> str:
    """Joins elements (converted by `to_string`) with `sep`.

## Parameters

  - xs: The elements to concatenate.
  - sep: Placed between elements.

    >>> join([1, 2, 3], "-")
    '1-2-3'
"""
    return sep.join(([str(x) for x in (xs)]))


def at(xs: list[_T_a], index: int) -> _T_a | None:
    """The element at `index` (negative counts from the end), or nil.

## Parameters

  - xs: The list to index.
  - index: Zero-based; negative counts from the end.

    >>> at([10, 20, 30], 1)
    20
"""
    if (index < builtins.len(xs)) and (index >= -(builtins.len(xs))):
        return ((xs)[(index)])
    else:
        return None


def take(xs: list[_T_a], n: int) -> list[_T_a]:
    """The first `n` elements.

## Parameters

  - xs: The source list.
  - n: How many leading elements to keep.

    >>> take([1, 2, 3], 2)
    [1, 2]
"""
    return ((xs)[:(n)])


def drop(xs: list[_T_a], n: int) -> list[_T_a]:
    """The elements after the first `n`.

## Parameters

  - xs: The source list.
  - n: How many leading elements to skip.

    >>> drop([1, 2, 3], 2)
    [3]
"""
    return ((xs)[(n):])


def zip(xs: list[_T_a], ys: list[_T_b]) -> list[tuple[_T_a, _T_b]]:
    """Pairs up two lists into `{a, b}` tuples, stopping at the shorter.

## Parameters

  - xs: The first list.
  - ys: The second list.

    >>> zip([1, 2], ["a", "b"])
    [(1, 'a'), (2, 'b')]
"""
    return builtins.list(builtins.zip(xs, ys))


def with_index(xs: list[_T_a]) -> list[tuple[_T_a, int]]:
    """Each element paired with its index: `{element, index}`.

## Parameters

  - xs: The list to enumerate.

    >>> with_index(["a", "b"])
    [('a', 0), ('b', 1)]
"""
    return ([(x, i) for i, x in enumerate((xs))])


def member_p(xs: list[_T_a], x: _T_a) -> bool:
    """Whether `x` is an element.

## Parameters

  - xs: The list to search.
  - x: The value to look for.

    >>> member_p([1, 2, 3], 2)
    True
"""
    return ((x) in (xs))


def all_p(xs: list[_T_a], f: collections.abc.Callable) -> bool:
    """Whether `f` is truthy for every element.

## Parameters

  - xs: The list to test.
  - f: The predicate every element must satisfy.

    >>> all_p([1, 2, 3], lambda x: x > 0)
    True
"""
    return (all((f)(x) for x in (xs)))


def any_p(xs: list[_T_a], f: collections.abc.Callable) -> bool:
    """Whether `f` is truthy for any element.

## Parameters

  - xs: The list to test.
  - f: The predicate at least one element must satisfy.

    >>> any_p([1, 2, 3], lambda x: x > 2)
    True
"""
    return (any((f)(x) for x in (xs)))


def empty_p(xs: list) -> bool:
    """Whether the collection has no elements.

## Parameters

  - xs: The list to test.

    >>> empty_p([])
    True
"""
    return builtins.len(xs) == 0


def uniq(xs: list[_T_a]) -> list[_T_a]:
    """Removes duplicates, keeping first occurrences in order.

## Parameters

  - xs: The list to deduplicate, keeping first occurrences.

    >>> uniq([1, 2, 1, 3, 2])
    [1, 2, 3]
"""
    return (list(dict.fromkeys((xs))))


def flat_map(xs: list[_T_a], f: collections.abc.Callable) -> list[_T_b]:
    """Maps `f` (which returns a list) and concatenates the results.

## Parameters

  - xs: The source list.
  - f: Returns a list per element; all are concatenated.

    >>> flat_map([1, 2], lambda x: [x, x * 10])
    [1, 10, 2, 20]
"""
    return ([y for x in (xs) for y in (f)(x)])


def each(xs: list[_T_a], f: collections.abc.Callable) -> str:
    """Runs `f` on each element for its side effects; returns :ok.

## Parameters

  - xs: The list to walk.
  - f: Called on each element for its side effect.

    >>> each(["a", "b"], lambda x: print(x))
    a
    b
    'ok'
"""
    ([(f)(x) for x in (xs)])
    return "ok"


def min(xs: list) -> object:
    """The smallest element.

## Parameters

  - xs: The non-empty list.

    >>> min([3, 1, 2])
    1
"""
    return builtins.min(xs)


def max(xs: list) -> object:
    """The largest element.

## Parameters

  - xs: The non-empty list.

    >>> max([3, 1, 2])
    3
"""
    return builtins.max(xs)


def find(xs: list[_T_a], f: collections.abc.Callable) -> _T_a | None:
    """The first element for which `f` is truthy, or nil.

## Parameters

  - xs: The list to search.
  - f: The predicate; the first truthy match is returned.

    >>> find([1, 2, 3, 4], lambda x: x > 2)
    3
"""
    return (next((x for x in (xs) if (f)(x)), None))


def find_index(xs: list, f: collections.abc.Callable) -> int | None:
    """The index of the first element for which `f` is truthy, or nil.

## Parameters

  - xs: The list to search.
  - f: The predicate; the index of the first match is returned.

    >>> find_index(["a", "b", "c"], lambda x: x == "b")
    1
"""
    return (next((i for i, x in enumerate((xs)) if (f)(x)), None))


def frequencies(xs: list[_T_a]) -> dict[_T_a, int]:
    """A map from each distinct element to its occurrence count.

## Parameters

  - xs: The elements to tally.

    >>> frequencies(["a", "b", "a"])
    {'a': 2, 'b': 1}
"""
    return builtins.dict(collections.Counter(xs))


def group_by(xs: list[_T_a], f: collections.abc.Callable) -> dict[_T_b, list[_T_a]]:
    """Groups elements by the key computed by `f`, preserving order.

## Parameters

  - xs: The elements to group.
  - f: Maps an element to its group key.

    >>> group_by([1, 2, 3, 4, 5], lambda x: _gan_rem(x, 2))
    {1: [1, 3, 5], 0: [2, 4]}
"""
    return ({k: [x for x in (xs) if (f)(x) == k] for k in dict.fromkeys((f)(x) for x in (xs))})


def max_by(xs: list[_T_a], f: collections.abc.Callable) -> _T_a:
    """The element maximizing the key computed by `f`; raises on empty.

## Parameters

  - xs: The non-empty list.
  - f: Maps an element to its comparison key.

    >>> max_by(["a", "bbb", "cc"], lambda s: gandora_std.string.length(s))
    'bbb'
"""
    return builtins.max(xs, key=f)


def min_by(xs: list[_T_a], f: collections.abc.Callable) -> _T_a:
    """The element minimizing the key computed by `f`; raises on empty.

## Parameters

  - xs: The non-empty list.
  - f: Maps an element to its comparison key.

    >>> min_by(["a", "bbb", "cc"], lambda s: gandora_std.string.length(s))
    'a'
"""
    return builtins.min(xs, key=f)


def product(xs: list) -> int | float:
    """The product of the elements (1 for an empty collection).

## Parameters

  - xs: The numbers to multiply.

    >>> product([2, 3, 4])
    24
"""
    return math.prod(xs)


def take_while(xs: list[_T_a], f: collections.abc.Callable) -> list[_T_a]:
    """Leading elements while `f` stays truthy.

## Parameters

  - xs: The source list.
  - f: Elements are taken while it returns truthy.

    >>> take_while([1, 2, 0, 3], lambda x: x > 0)
    [1, 2]
"""
    return builtins.list(itertools.takewhile(f, xs))


def drop_while(xs: list[_T_a], f: collections.abc.Callable) -> list[_T_a]:
    """Drops leading elements while `f` stays truthy, keeps the rest.

## Parameters

  - xs: The source list.
  - f: Elements are dropped while it returns truthy.

    >>> drop_while([1, 2, 0, 3], lambda x: x > 0)
    [0, 3]
"""
    return builtins.list(itertools.dropwhile(f, xs))


def chunk_every(xs: list[_T_a], n: int) -> list[list[_T_a]]:
    """Splits into chunks of `n` elements; the last chunk may be shorter.

## Parameters

  - xs: The list to split.
  - n: The chunk size; the last chunk may be shorter.

    >>> chunk_every([1, 2, 3, 4, 5], 2)
    [[1, 2], [3, 4], [5]]
"""
    return ([(xs)[i:i + (n)] for i in range(0, len((xs)), (n))])


def concat(xss: list[list[_T_a]]) -> list[_T_a]:
    """Concatenates a list of lists (one level).

## Parameters

  - xss: A list of lists to flatten one level.

    >>> concat([[1, 2], [3], []])
    [1, 2, 3]
"""
    return ([y for x in (xss) for y in x])


def intersperse(xs: list[_T_a], sep: _T_a) -> list[_T_a]:
    """Puts `sep` between every two elements.

## Parameters

  - xs: The source list.
  - sep: Inserted between every two elements.

    >>> intersperse([1, 2, 3], "x")
    [1, 'x', 2, 'x', 3]
"""
    return ([v for x in (xs) for v in (x, (sep))][:-1])


def slice(xs: list[_T_a], start: int, count: int) -> list[_T_a]:
    """`count` elements starting at `start` (negative start counts from the end).

## Parameters

  - xs: The source list.
  - start: Zero-based start position.
  - count: Maximum number of elements.

    >>> slice([1, 2, 3, 4], 1, 2)
    [2, 3]
"""
    return ((xs)[(start):] [:(count)])


def dedup(xs: list[_T_a]) -> list[_T_a]:
    """Collapses consecutive duplicate elements.

## Parameters

  - xs: The list whose adjacent duplicates collapse.

    >>> dedup([1, 1, 2, 2, 2, 1])
    [1, 2, 1]
"""
    return ([x for i, x in enumerate((xs)) if i == 0 or x != (xs)[i - 1]])
