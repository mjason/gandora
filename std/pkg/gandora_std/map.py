"""Data-first dict functions; all updates return new maps (GEP-0010)."""

import builtins


def _gan_truthy(value):
    return value is not None and value is not False

class GanMatchError(Exception):
    pass


def get(*_gan_args):
    """The value for `key`, or `default` (nil unless given) when absent.


      >>> get({"a": 1}, "b", 0)
      0
"""
    match _gan_args:
        case (m, key, default,):
            return m.get(key, default)
        case (m, key,):
            return get(m, key, None)
    raise GanMatchError("no clause of get/2,3 matched " + repr(_gan_args))


def put(m, key, value):
    """A new map with `key` set to `value`.


      >>> put({"a": 1}, "b", 2)
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


      >>> update({"a": 1}, "a", 0, lambda v: v + 10)
      {'a': 11}
"""
    if _gan_truthy(has_key_p(m, key)):
        return put(m, key, f(get(m, key)))
    else:
        return put(m, key, default)


def fetch_bang(m, key):
    """The value for `key`; raises Python KeyError when absent."""
    return ((m)[(key)])


def put_new(m, key, value):
    """Sets `key` only when absent."""
    if _gan_truthy(has_key_p(m, key)):
        return m
    else:
        return put(m, key, value)


def take(m, keys):
    """The submap with only `keys` (missing keys ignored)."""
    return ({k: v for k, v in (m).items() if k in (keys)})


def drop(m, keys):
    """The map without `keys`."""
    return ({k: v for k, v in (m).items() if k not in (keys)})


def filter(m, f):
    """Keeps entries for which `f` (receiving a `{key, value}` tuple) is truthy.


      >>> filter({"a": 1, "b": 5}, lambda pair: pair[1] > 2)
      {'b': 5}
"""
    return ({k: v for k, v in (m).items() if (f)((k, v))})
