"""Data-first dict functions; all updates return new maps (GEP-0010)."""

import builtins
import gandora_std.map


def _gan_truthy(value):
    return value is not None and value is not False


def get(m, key):
    """The value for `key`, or nil when absent."""
    return m.get(key)


def put(m, key, value):
    """A new map with `key` set to `value`.


      >>> gandora_std.map.put({"a": 1}, "b", 2)
      {'a': 1, 'b': 2}
"""
    return ({**(m), (key): (value)})


def delete(m, key):
    """A new map without `key`."""
    return ({k: v for k, v in (m).items() if k != (key)})


def keys(m):
    """The keys as a list."""
    return builtins.list(m.keys())


def values(m):
    """The values as a list."""
    return builtins.list(m.values())


def merge(m1, m2):
    """Merges `m2` into `m1`; `m2` wins on conflicts."""
    return ({**(m1), **(m2)})


def has_key_p(m, key):
    """Whether `key` is present."""
    return ((key) in (m))


def to_list(m):
    """The entries as a list of `{key, value}` tuples."""
    return builtins.list(m.items())


def new():
    """An empty map."""
    return {}


def update(m, key, default, f):
    """Updates `key` by `f`; uses `default` when the key is absent (Elixir Map.update/4).


      >>> gandora_std.map.update({"a": 1}, "a", 0, lambda v: v + 10)
      {'a': 11}
"""
    if _gan_truthy(has_key_p(m, key)):
        return put(m, key, f(get(m, key)))
    else:
        return put(m, key, default)
