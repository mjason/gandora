"""Python interop: atom calls, pyimport, postfix chains, kwargs, decorators."""

import collections
import functools
import json as j
import math
import os.path

counter = collections.Counter("gandora")


@functools.lru_cache(maxsize=8)
def cached_add(a, b):
    return a + b


def demo():
    print(f":math.pi           = {math.pi}")
    _gan_fstr0 = j.dumps({"lang": "gandora", "year": 2026}, sort_keys=True)
    print(f"json with kwargs   = {_gan_fstr0}")
    print(f"counter.most_common = {repr(counter.most_common(2))}")
    _gan_fstr1 = os.path.join("a", "b")
    print(f"dotted module      = {_gan_fstr1}")
    _gan_fstr2 = " gandora ".strip().upper()
    print(f"postfix chain      = {_gan_fstr2}")
    return print(f"cached_add(20, 22) = {cached_add(20, 22)}")
