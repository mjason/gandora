"""Functions over keyword lists — lists of {atom, value} tuples (GEP-0010)."""

import gandora_std.enum


class GanMatchError(Exception):
    pass


def get(kw: list[tuple[str, object]], key: str) -> object:
    """The first value for `key`, or nil.

## Parameters

  - kw: The keyword list.
  - key: The key to read; first match wins.

    >>> get([("a", 1), ("b", 2)], "a")
    1
"""
    return (next((v for k, v in (kw) if k == (key)), None))


def put(kw: list[tuple[str, object]], key: str, value: object) -> list[tuple[str, object]]:
    """Replaces `key` with `value`, prepending it (Elixir Keyword.put).

## Parameters

  - kw: The keyword list.
  - key: The key to set.
  - value: The value to store.
"""
    def _gan_fn0(*_gan_args, key=key):
        match _gan_args:
            case ((k, _) as _gan_t0,) if isinstance(_gan_t0, tuple):
                return k == key
        raise GanMatchError("no clause of _gan_fn0/1 matched " + repr(_gan_args))
    return [(key, value)] + gandora_std.enum.reject(kw, _gan_fn0)


def keys(kw: list[tuple[str, object]]) -> list[str]:
    """The keys, in order, duplicates included.

## Parameters

  - kw: The keyword list.
"""
    def _gan_fn1(*_gan_args):
        match _gan_args:
            case ((k, _) as _gan_t1,) if isinstance(_gan_t1, tuple):
                return k
        raise GanMatchError("no clause of _gan_fn1/1 matched " + repr(_gan_args))
    return gandora_std.enum.map(kw, _gan_fn1)


def values(kw: list[tuple[str, object]]) -> list:
    """The values, in order.

## Parameters

  - kw: The keyword list.
"""
    def _gan_fn2(*_gan_args):
        match _gan_args:
            case ((_, v) as _gan_t2,) if isinstance(_gan_t2, tuple):
                return v
        raise GanMatchError("no clause of _gan_fn2/1 matched " + repr(_gan_args))
    return gandora_std.enum.map(kw, _gan_fn2)


def has_key_p(kw: list[tuple[str, object]], key: str) -> bool:
    """Whether `key` is present.

## Parameters

  - kw: The keyword list.
  - key: The key to test.
"""
    def _gan_fn3(*_gan_args, key=key):
        match _gan_args:
            case ((k, _) as _gan_t3,) if isinstance(_gan_t3, tuple):
                return k == key
        raise GanMatchError("no clause of _gan_fn3/1 matched " + repr(_gan_args))
    return gandora_std.enum.any_p(kw, _gan_fn3)


def delete(kw: list[tuple[str, object]], key: str) -> list[tuple[str, object]]:
    """Removes every entry for `key`.

## Parameters

  - kw: The keyword list.
  - key: Every entry with this key is removed.
"""
    def _gan_fn4(*_gan_args, key=key):
        match _gan_args:
            case ((k, _) as _gan_t4,) if isinstance(_gan_t4, tuple):
                return k == key
        raise GanMatchError("no clause of _gan_fn4/1 matched " + repr(_gan_args))
    return gandora_std.enum.reject(kw, _gan_fn4)


def merge(kw1: list[tuple[str, object]], kw2: list[tuple[str, object]]) -> list[tuple[str, object]]:
    """Merges `kw2` into `kw1`; `kw2` wins, its entries appended.

## Parameters

  - kw1: The base list.
  - kw2: Its entries win on conflicts.

    >>> merge([("a", 1), ("b", 2)], [("b", 9), ("c", 3)])
    [('a', 1), ('b', 9), ('c', 3)]
"""
    def _gan_fn5(*_gan_args, kw2=kw2):
        match _gan_args:
            case ((k, _) as _gan_t5,) if isinstance(_gan_t5, tuple):
                return has_key_p(kw2, k)
        raise GanMatchError("no clause of _gan_fn5/1 matched " + repr(_gan_args))
    return gandora_std.enum.reject(kw1, _gan_fn5) + kw2
