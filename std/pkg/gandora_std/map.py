"""Data-first dict functions; all updates return new maps (GEP-0010)."""

import builtins
import collections.abc
import typing


def _gan_truthy(value):
    return value is not None and value is not False

class GanMatchError(Exception):
    pass

_T_d = typing.TypeVar("_T_d")
_T_k = typing.TypeVar("_T_k")
_T_v = typing.TypeVar("_T_v")


def get(*_gan_args) -> _T_v | _T_d:
    """The value for `key`, or `default` (nil unless given) when absent.

## Parameters

  - m: The map.
  - key: The key to read.
  - default: Returned when the key is absent.

    >>> get({"a": 1}, "b", 0)
    0
"""
    while True:
        match _gan_args:
            case (m, key, default,):
                return m.get(key, default)
            case (m, key,):
                _gan_args = (m, key, None)
                continue
        raise GanMatchError("no clause of get/2,3 matched " + repr(_gan_args))


def put(m: dict[_T_k, _T_v], key: _T_k, value: _T_v) -> dict[_T_k, _T_v]:
    """A new map with `key` set to `value`.

## Parameters

  - m: The map.
  - key: The key to set.
  - value: The value to store.

    >>> put({"a": 1}, "b", 2)
    {'a': 1, 'b': 2}
"""
    return ({**(m), (key): (value)})


def delete(m: dict[_T_k, _T_v], key: _T_k) -> dict[_T_k, _T_v]:
    """A new map without `key`.

## Parameters

  - m: The map.
  - key: The key to remove.

    >>> delete({"a": 1, "b": 2}, "a")
    {'b': 2}
"""
    return ({k: v for k, v in (m).items() if k != (key)})


def keys(m: dict[_T_k, _T_v]) -> list[_T_k]:
    """The keys as a list.

## Parameters

  - m: The map.

    >>> keys({"a": 1, "b": 2})
    ['a', 'b']
"""
    return builtins.list(m.keys())


def values(m: dict[_T_k, _T_v]) -> list[_T_v]:
    """The values as a list.

## Parameters

  - m: The map.

    >>> values({"a": 1, "b": 2})
    [1, 2]
"""
    return builtins.list(m.values())


def merge(m1: dict[_T_k, _T_v], m2: dict[_T_k, _T_v]) -> dict[_T_k, _T_v]:
    """Merges `m2` into `m1`; `m2` wins on conflicts.

## Parameters

  - m1: The base map.
  - m2: Its entries win on conflicts.

    >>> merge({"a": 1}, {"b": 2})
    {'a': 1, 'b': 2}
"""
    return ({**(m1), **(m2)})


def has_key_p(m: dict[_T_k, _T_v], key: _T_k) -> bool:
    """Whether `key` is present.

## Parameters

  - m: The map.
  - key: The key to test.

    >>> has_key_p({"a": 1}, "a")
    True
"""
    return ((key) in (m))


def to_list(m: dict[_T_k, _T_v]) -> list[tuple[_T_k, _T_v]]:
    """The entries as a list of `{key, value}` tuples.

## Parameters

  - m: The map.

    >>> to_list({"a": 1, "b": 2})
    [('a', 1), ('b', 2)]
"""
    return builtins.list(m.items())


def new(*_gan_args) -> dict:
    """An empty map.

    >>> new([("a", 1), ("b", 2)])
    {'a': 1, 'b': 2}
"""
    match _gan_args:
        case ():
            return {}
        case (pairs,):
            return builtins.dict(pairs)
    raise GanMatchError("no clause of new/0,1 matched " + repr(_gan_args))


def update(m: dict[_T_k, _T_v], key: _T_k, default: _T_v, f: collections.abc.Callable) -> dict[_T_k, _T_v]:
    """Updates `key` by `f`; uses `default` when the key is absent (Elixir Map.update/4).

## Parameters

  - m: The map.
  - key: The key to update.
  - default: Stored when the key is absent.
  - f: Applied to the current value when present.

    >>> update({"a": 1}, "a", 0, lambda v: v + 10)
    {'a': 11}
"""
    if _gan_truthy(has_key_p(m, key)):
        return put(m, key, f(get(m, key)))
    else:
        return put(m, key, default)


def fetch_bang(m: dict[_T_k, _T_v], key: _T_k) -> _T_v:
    """The value for `key`; raises Python KeyError when absent.

## Parameters

  - m: The map.
  - key: The key that must exist.

    >>> fetch_bang({"a": 1}, "a")
    1
"""
    return ((m)[(key)])


def put_new(m: dict[_T_k, _T_v], key: _T_k, value: _T_v) -> dict[_T_k, _T_v]:
    """Sets `key` only when absent.

## Parameters

  - m: The map.
  - key: The key to set only if absent.
  - value: The value to store.

    >>> put_new({"a": 1}, "a", 9)
    {'a': 1}
"""
    if _gan_truthy(has_key_p(m, key)):
        return m
    else:
        return put(m, key, value)


def take(m: dict[_T_k, _T_v], keys: list[_T_k]) -> dict[_T_k, _T_v]:
    """The submap with only `keys` (missing keys ignored).

## Parameters

  - m: The map.
  - keys: The keys to keep.

    >>> take({"a": 1, "b": 2}, ["a"])
    {'a': 1}
"""
    return ({k: v for k, v in (m).items() if k in (keys)})


def drop(m: dict[_T_k, _T_v], keys: list[_T_k]) -> dict[_T_k, _T_v]:
    """The map without `keys`.

## Parameters

  - m: The map.
  - keys: The keys to remove.

    >>> drop({"a": 1, "b": 2}, ["a"])
    {'b': 2}
"""
    return ({k: v for k, v in (m).items() if k not in (keys)})


def filter(m: dict[_T_k, _T_v], f: collections.abc.Callable) -> dict[_T_k, _T_v]:
    """Keeps entries for which `f` (receiving a `{key, value}` tuple) is truthy.

## Parameters

  - m: The map.
  - f: Receives {key, value}; keeps entries it returns truthy for.

    >>> filter({"a": 1, "b": 5}, lambda pair: pair[1] > 2)
    {'b': 5}
"""
    return ({k: v for k, v in (m).items() if (f)((k, v))})
