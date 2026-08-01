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
    print(f"json with kwargs   = {j.dumps({"lang": "gandora", "year": 2026}, sort_keys=True)}")
    print(f"counter.most_common = {repr(counter.most_common(2))}")
    print(f"dotted module      = {os.path.join("a", "b")}")
    print(f"postfix chain      = {" gandora ".strip().upper()}")
    return print(f"cached_add(20, 22) = {cached_add(20, 22)}")
