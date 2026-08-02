"""Wrapping a native Python decorator as an Elixir-style attribute — the
same mechanism @doc itself uses (GEP-0008). `@cache 64` before a def
applies functools.lru_cache(maxsize: 64); functions without the
attribute pass through untouched.
"""

import builtins
import functools


@functools.lru_cache(maxsize=128)
def fib(n):
    if n < 2:
        return n
    else:
        return fib(n - 1) + fib(n - 2)


def plain(x):
    return x + 1


def main():
    print(f"fib(60) = {fib(60)}")
    print(f"cache   = {fib.cache_info()}")
    _gan_fstr0 = repr(builtins.hasattr(plain, "cache_info"))
    return print(f"plain has no cache_info: {_gan_fstr0}")


if __name__ == "__main__":
    main()
