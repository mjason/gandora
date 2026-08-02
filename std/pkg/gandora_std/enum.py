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

## Parameters

  - xs: The source list.
  - f: Applied to each element; the results form the new list.

    >>> map([1, 2, 3], lambda x: x * 10)
    [10, 20, 30]
"""
    return builtins.list(builtins.map(f, xs))


def filter(xs: list, f: collections.abc.Callable) -> list:
    """Keeps elements for which `f` is truthy.

## Parameters

  - xs: The source list.
  - f: Keeps the elements it returns truthy for.

    >>> filter([1, 2, 3, 4], lambda x: _gan_rem(x, 2) == 0)
    [2, 4]
"""
    return ([x for x in (xs) if (f)(x)])


def reject(xs: list, f: collections.abc.Callable) -> list:
    """Drops elements for which `f` is truthy.

## Parameters

  - xs: The source list.
  - f: Drops the elements it returns truthy for.
"""
    return ([x for x in (xs) if not (f)(x)])


def reduce(xs: list, acc: object, f: collections.abc.Callable) -> object:
    """Folds left with Elixir argument order: `f` receives (element, acc).

## Parameters

  - xs: The list to fold.
  - acc: The starting accumulator.
  - f: Receives an element and the accumulator, returns the next accumulator.

    >>> reduce([1, 2, 3, 4], 0, lambda x, acc: acc + x)
    10
"""
    return functools.reduce(lambda a, x: f(x, a), xs, acc)


def sum(xs: list) -> int | float:
    """Sum of the elements.

## Parameters

  - xs: The numbers to add.
"""
    return builtins.sum(xs)


def count(xs: list) -> int:
    """Number of elements.

## Parameters

  - xs: The list to measure.
"""
    return builtins.len(xs)


def sort(xs: list) -> list:
    """Ascending sort.

## Parameters

  - xs: The list to order ascending.
"""
    return builtins.sorted(xs)


def sort_by(xs: list, f: collections.abc.Callable) -> list:
    """Sorts by the key computed by `f` (Python `key=`, not a comparator).

## Parameters

  - xs: The list to order.
  - f: Maps each element to its sort key.

    >>> sort_by(["ccc", "a", "bb"], lambda s: gandora_std.string.length(s))
    ['a', 'bb', 'ccc']
"""
    return builtins.sorted(xs, key=f)


def reverse(xs: list) -> list:
    """Elements in reverse order.

## Parameters

  - xs: The list to reverse.
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


def at(xs: list, index: int) -> object:
    """The element at `index` (negative counts from the end), or nil.

## Parameters

  - xs: The list to index.
  - index: Zero-based; negative counts from the end.
"""
    if (index < builtins.len(xs)) and (index >= -(builtins.len(xs))):
        return ((xs)[(index)])
    else:
        return None


def take(xs: list, n: int) -> list:
    """The first `n` elements.

## Parameters

  - xs: The source list.
  - n: How many leading elements to keep.
"""
    return ((xs)[:(n)])


def drop(xs: list, n: int) -> list:
    """The elements after the first `n`.

## Parameters

  - xs: The source list.
  - n: How many leading elements to skip.
"""
    return ((xs)[(n):])


def zip(xs: list, ys: list) -> list[tuple[object, object]]:
    """Pairs up two lists into `{a, b}` tuples, stopping at the shorter.

## Parameters

  - xs: The first list.
  - ys: The second list.
"""
    return builtins.list(builtins.zip(xs, ys))


def with_index(xs: list) -> list[tuple[object, int]]:
    """Each element paired with its index: `{element, index}`.

## Parameters

  - xs: The list to enumerate.

    >>> with_index(["a", "b"])
    [('a', 0), ('b', 1)]
"""
    return ([(x, i) for i, x in enumerate((xs))])


def member_p(xs: list, x: object) -> bool:
    """Whether `x` is an element.

## Parameters

  - xs: The list to search.
  - x: The value to look for.
"""
    return ((x) in (xs))


def all_p(xs: list, f: collections.abc.Callable) -> bool:
    """Whether `f` is truthy for every element.

## Parameters

  - xs: The list to test.
  - f: The predicate every element must satisfy.
"""
    return (all((f)(x) for x in (xs)))


def any_p(xs: list, f: collections.abc.Callable) -> bool:
    """Whether `f` is truthy for any element.

## Parameters

  - xs: The list to test.
  - f: The predicate at least one element must satisfy.
"""
    return (any((f)(x) for x in (xs)))


def empty_p(xs: list) -> bool:
    """Whether the collection has no elements.

## Parameters

  - xs: The list to test.
"""
    return builtins.len(xs) == 0


def uniq(xs: list) -> list:
    """Removes duplicates, keeping first occurrences in order.

## Parameters

  - xs: The list to deduplicate, keeping first occurrences.
"""
    return (list(dict.fromkeys((xs))))


def flat_map(xs: list, f: collections.abc.Callable) -> list:
    """Maps `f` (which returns a list) and concatenates the results.

## Parameters

  - xs: The source list.
  - f: Returns a list per element; all are concatenated.

    >>> flat_map([1, 2], lambda x: [x, x * 10])
    [1, 10, 2, 20]
"""
    return ([y for x in (xs) for y in (f)(x)])


def each(xs: list, f: collections.abc.Callable) -> str:
    """Runs `f` on each element for its side effects; returns :ok.

## Parameters

  - xs: The list to walk.
  - f: Called on each element for its side effect.
"""
    ([(f)(x) for x in (xs)])
    return "ok"


def min(xs: list) -> object:
    """The smallest element.

## Parameters

  - xs: The non-empty list.
"""
    return builtins.min(xs)


def max(xs: list) -> object:
    """The largest element.

## Parameters

  - xs: The non-empty list.
"""
    return builtins.max(xs)


def find(xs: list, f: collections.abc.Callable) -> object:
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
"""
    return (next((i for i, x in enumerate((xs)) if (f)(x)), None))


def frequencies(xs: list) -> dict[object, int]:
    """A map from each distinct element to its occurrence count.

## Parameters

  - xs: The elements to tally.

    >>> frequencies(["a", "b", "a"])
    {'a': 2, 'b': 1}
"""
    return builtins.dict(collections.Counter(xs))


def group_by(xs: list, f: collections.abc.Callable) -> dict:
    """Groups elements by the key computed by `f`, preserving order.

## Parameters

  - xs: The elements to group.
  - f: Maps an element to its group key.

    >>> group_by([1, 2, 3, 4, 5], lambda x: _gan_rem(x, 2))
    {1: [1, 3, 5], 0: [2, 4]}
"""
    return ({k: [x for x in (xs) if (f)(x) == k] for k in dict.fromkeys((f)(x) for x in (xs))})


def max_by(xs: list, f: collections.abc.Callable) -> object:
    """The element maximizing the key computed by `f`; raises on empty.

## Parameters

  - xs: The non-empty list.
  - f: Maps an element to its comparison key.
"""
    return builtins.max(xs, key=f)


def min_by(xs: list, f: collections.abc.Callable) -> object:
    """The element minimizing the key computed by `f`; raises on empty.

## Parameters

  - xs: The non-empty list.
  - f: Maps an element to its comparison key.
"""
    return builtins.min(xs, key=f)


def product(xs: list) -> int | float:
    """The product of the elements (1 for an empty collection).

## Parameters

  - xs: The numbers to multiply.
"""
    return math.prod(xs)


def take_while(xs: list, f: collections.abc.Callable) -> list:
    """Leading elements while `f` stays truthy.

## Parameters

  - xs: The source list.
  - f: Elements are taken while it returns truthy.
"""
    return builtins.list(itertools.takewhile(f, xs))


def drop_while(xs: list, f: collections.abc.Callable) -> list:
    """Drops leading elements while `f` stays truthy, keeps the rest.

## Parameters

  - xs: The source list.
  - f: Elements are dropped while it returns truthy.
"""
    return builtins.list(itertools.dropwhile(f, xs))


def chunk_every(xs: list, n: int) -> list[list]:
    """Splits into chunks of `n` elements; the last chunk may be shorter.

## Parameters

  - xs: The list to split.
  - n: The chunk size; the last chunk may be shorter.

    >>> chunk_every([1, 2, 3, 4, 5], 2)
    [[1, 2], [3, 4], [5]]
"""
    return ([(xs)[i:i + (n)] for i in range(0, len((xs)), (n))])


def concat(xss: list[list]) -> list:
    """Concatenates a list of lists (one level).

## Parameters

  - xss: A list of lists to flatten one level.
"""
    return ([y for x in (xss) for y in x])


def intersperse(xs: list, sep: object) -> list:
    """Puts `sep` between every two elements.

## Parameters

  - xs: The source list.
  - sep: Inserted between every two elements.

    >>> intersperse([1, 2, 3], "x")
    [1, 'x', 2, 'x', 3]
"""
    return ([v for x in (xs) for v in (x, (sep))][:-1])


def slice(xs: list, start: int, count: int) -> list:
    """`count` elements starting at `start` (negative start counts from the end).

## Parameters

  - xs: The source list.
  - start: Zero-based start position.
  - count: Maximum number of elements.
"""
    return ((xs)[(start):] [:(count)])


def dedup(xs: list) -> list:
    """Collapses consecutive duplicate elements.

## Parameters

  - xs: The list whose adjacent duplicates collapse.

    >>> dedup([1, 1, 2, 2, 2, 1])
    [1, 2, 1]
"""
    return ([x for i, x in enumerate((xs)) if i == 0 or x != (xs)[i - 1]])
