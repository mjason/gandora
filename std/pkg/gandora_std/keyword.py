"""Functions over keyword lists — lists of {atom, value} tuples (GEP-0010)."""

import gandora_std.enum


class GanMatchError(Exception):
    pass


def get(kw, key):
    """The first value for `key`, or nil.


      >>> get([("a", 1), ("b", 2)], "a")
      1
"""
    return (next((v for k, v in (kw) if k == (key)), None))


def put(kw, key, value):
    """Replaces `key` with `value`, prepending it (Elixir Keyword.put)."""
    def _gan_fn0(*_gan_args):
        match _gan_args:
            case ((k, _) as _gan_t0,) if isinstance(_gan_t0, tuple):
                return k == key
        raise GanMatchError("no clause of _gan_fn0/1 matched " + repr(_gan_args))
    return [(key, value)] + gandora_std.enum.reject(kw, _gan_fn0)


def keys(kw):
    """The keys, in order, duplicates included."""
    def _gan_fn1(*_gan_args):
        match _gan_args:
            case ((k, _) as _gan_t1,) if isinstance(_gan_t1, tuple):
                return k
        raise GanMatchError("no clause of _gan_fn1/1 matched " + repr(_gan_args))
    return gandora_std.enum.map(kw, _gan_fn1)


def values(kw):
    """The values, in order."""
    def _gan_fn2(*_gan_args):
        match _gan_args:
            case ((_, v) as _gan_t2,) if isinstance(_gan_t2, tuple):
                return v
        raise GanMatchError("no clause of _gan_fn2/1 matched " + repr(_gan_args))
    return gandora_std.enum.map(kw, _gan_fn2)


def has_key_p(kw, key):
    """Whether `key` is present."""
    def _gan_fn3(*_gan_args):
        match _gan_args:
            case ((k, _) as _gan_t3,) if isinstance(_gan_t3, tuple):
                return k == key
        raise GanMatchError("no clause of _gan_fn3/1 matched " + repr(_gan_args))
    return gandora_std.enum.any_p(kw, _gan_fn3)
