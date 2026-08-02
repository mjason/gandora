"""Data-first dict functions; all updates return new maps (GEP-0010)."""

import builtins
import collections.abc


def _gan_truthy(value):
    return value is not None and value is not False

class GanMatchError(Exception):
    pass


def get(*_gan_args) -> object:
    """The value for `key`, or `default` (nil unless given) when absent.

## Parameters

  - m: The map.
  - key: The key to read.
  - default: Returned when the key is absent.

    >>> get({"a": 1}, "b", 0)
    0
"""
    match _gan_args:
        case (m, key, default,):
            return m.get(key, default)
        case (m, key,):
            return get(m, key, None)
    raise GanMatchError("no clause of get/2,3 matched " + repr(_gan_args))


def put(m: dict, key: object, value: object) -> dict:
    """A new map with `key` set to `value`.

## Parameters

  - m: The map.
  - key: The key to set.
  - value: The value to store.

    >>> put({"a": 1}, "b", 2)
    {'a': 1, 'b': 2}
"""
    return ({**(m), (key): (value)})


def delete(m: dict, key: object) -> dict:
    """A new map without `key`.

## Parameters

  - m: The map.
  - key: The key to remove.
"""
    return ({k: v for k, v in (m).items() if k != (key)})


def keys(m: dict) -> list:
    """The keys as a list.

## Parameters

  - m: The map.
"""
    return builtins.list(m.keys())


def values(m: dict) -> list:
    """The values as a list.

## Parameters

  - m: The map.
"""
    return builtins.list(m.values())


def merge(m1: dict, m2: dict) -> dict:
    """Merges `m2` into `m1`; `m2` wins on conflicts.

## Parameters

  - m1: The base map.
  - m2: Its entries win on conflicts.
"""
    return ({**(m1), **(m2)})


def has_key_p(m: dict, key: object) -> bool:
    """Whether `key` is present.

## Parameters

  - m: The map.
  - key: The key to test.
"""
    return ((key) in (m))


def to_list(m: dict) -> list[tuple[object, object]]:
    """The entries as a list of `{key, value}` tuples.

## Parameters

  - m: The map.
"""
    return builtins.list(m.items())


def new() -> dict:
    """An empty map."""
    return {}


def update(m: dict, key: object, default: object, f: collections.abc.Callable) -> dict:
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


def fetch_bang(m: dict, key: object) -> object:
    """The value for `key`; raises Python KeyError when absent.

## Parameters

  - m: The map.
  - key: The key that must exist.
"""
    return ((m)[(key)])


def put_new(m: dict, key: object, value: object) -> dict:
    """Sets `key` only when absent.

## Parameters

  - m: The map.
  - key: The key to set only if absent.
  - value: The value to store.
"""
    if _gan_truthy(has_key_p(m, key)):
        return m
    else:
        return put(m, key, value)


def take(m: dict, keys: list) -> dict:
    """The submap with only `keys` (missing keys ignored).

## Parameters

  - m: The map.
  - keys: The keys to keep.
"""
    return ({k: v for k, v in (m).items() if k in (keys)})


def drop(m: dict, keys: list) -> dict:
    """The map without `keys`.

## Parameters

  - m: The map.
  - keys: The keys to remove.
"""
    return ({k: v for k, v in (m).items() if k not in (keys)})


def filter(m: dict, f: collections.abc.Callable) -> dict:
    """Keeps entries for which `f` (receiving a `{key, value}` tuple) is truthy.

## Parameters

  - m: The map.
  - f: Receives {key, value}; keeps entries it returns truthy for.

    >>> filter({"a": 1, "b": 5}, lambda pair: pair[1] > 2)
    {'b': 5}
"""
    return ({k: v for k, v in (m).items() if (f)((k, v))})
