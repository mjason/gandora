"""Anonymous functions, captures, and data-first pipelines."""

import builtins
import math


def _gan_div(a, b):
    q = a // b
    if q < 0 and q * b != a:
        q += 1
    return q


def _gan_rem(a, b):
    return a - _gan_div(a, b) * b

class GanMatchError(Exception):
    pass


def _map_list(xs, f):
    return builtins.list(builtins.map(f, xs))


def _keep(xs, f):
    return builtins.list(builtins.filter(f, xs))


def demo() -> None:
    """Runs the chapter."""
    double = lambda x: x * 2
    def _gan_fn0(*_gan_args):
        match _gan_args:
            case (0,):
                return "zero"
            case (n,) if n > 0:
                return "pos"
            case (_,):
                return "neg"
        raise GanMatchError("no clause of _gan_fn0/1 matched " + repr(_gan_args))
    classify = _gan_fn0
    add = lambda _gan_cap1, _gan_cap2: _gan_cap1 + _gan_cap2
    sqrt = math.sqrt
    print(f"double.(21)  = {double(21)}")
    print(f"classify     = {repr([classify(0), classify(5), classify(-5)])}")
    print(f"add.(40, 2)  = {add(40, 2)}")
    print(f"sqrt.(81.0)  = {sqrt(81.0)}")
    result = builtins.sum(_keep(_map_list(builtins.list(range(1, (10) + 1)), lambda x: x * x), lambda x: _gan_rem(x, 2) == 0))
    return print(f"1..10 squared, evens only, summed = {result}")
