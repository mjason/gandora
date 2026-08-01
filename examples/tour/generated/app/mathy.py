"""Math helpers showing patterns, interop, and documented examples."""

import functools
import math


class GanMatchError(Exception):
    pass


def fact(*_gan_args):
    """
  Factorial via multi-clause dispatch. Prose lives in @doc and is
  translated; runnable examples live in @example, tested by `gan test`
  and shared by every locale.


  ## Examples

      >>> fact(0)
      1
      >>> fact(10)
      3628800

Since: 0.1.0
"""
    match _gan_args:
        case (0,):
            return 1
        case (n,) if n > 0:
            return n * fact(n - 1)
    raise GanMatchError("no clause of fact/1 matched " + repr(_gan_args))


def classify(x):
    """Sign of a number as an atom.


      >>> classify(-3)
      'negative'
      >>> [classify(0), classify(9)]
      ['zero', 'positive']
"""
    if x < 0:
        return "negative"
    elif x == 0:
        return "zero"
    else:
        return "positive"


def old_norm(xs):
    """Same as norm/1; kept for early adopters.

Deprecated: Use norm/1 instead.

Since: 0.1.0
"""
    return norm(xs)


def norm(xs):
    """Euclidean norm through a pipeline.


      >>> norm([3, 4])
      5.0

Since: 0.1.0
"""
    return _then_sqrt(_sum_squares(xs))


def _sum_squares(xs):
    total = functools.reduce(lambda acc, x: acc + (x * x), xs, 0)
    return total


def _then_sqrt(x):
    return math.sqrt(x)
