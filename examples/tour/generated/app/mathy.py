"""Math helpers showing patterns and interop."""

import functools
import math


class GanMatchError(Exception):
    pass


def fact(*_gan_args):
    """Truncated factorial via multi-clause dispatch."""
    match _gan_args:
        case (0,):
            return 1
        case (n,) if n > 0:
            return n * fact(n - 1)
    raise GanMatchError("no clause of fact/1 matched " + repr(_gan_args))


def classify(x):
    if x < 0:
        return "negative"
    elif x == 0:
        return "zero"
    else:
        return "positive"


def norm(xs):
    return _then_sqrt(_sum_squares(xs))


def _sum_squares(xs):
    total = functools.reduce(lambda acc, x: acc + (x * x), xs, 0)
    return total


def _then_sqrt(x):
    return math.sqrt(x)
