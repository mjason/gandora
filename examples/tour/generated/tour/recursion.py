"""Iteration the Gandora way (GEP-0019/0020): tail recursion compiles
to a `while` loop and runs in constant stack, `recur` is the
compile-checked spelling of that jump, structural recursion says so
with `@allow`, and `for` comprehensions compile to native Python
comprehensions. Hover any head in your editor to see the compiled
shape; `gan doc Tour.Recursion.sum_to` prints it too.
"""

import builtins


def _gan_div(a, b):
    q = a // b
    if q < 0 and q * b != a:
        q += 1
    return q


def _gan_rem(a, b):
    return a - _gan_div(a, b) * b

class GanMatchError(Exception):
    pass


def sum_to(*_gan_args) -> int:
    """Sums `1..n` — the tail call rebinds parameters, no stack growth.

## Parameters

  - n: The upper bound; a million frames is fine.
"""
    while True:
        match _gan_args:
            case (n,):
                _gan_args = (n, 0)
                continue
            case (0, acc,):
                return acc
            case (n, acc,):
                _gan_args = (n - 1, acc + n)
                continue
        raise GanMatchError("no clause of sum_to/1,2 matched " + repr(_gan_args))


def countdown(n: int) -> str:
    """Counts down using `recur` — the explicit spelling: the compiler
rejects it outside tail position or with a wrong arity.

## Parameters

  - n: Where the countdown starts.
"""
    while True:
        if n <= 0:
            return "done"
        else:
            n = n - 1
            continue


def depth(*_gan_args) -> int:
    """Nesting depth of a list — structural recursion: the depth is bounded
by the data's shape, and `@allow :stack_recursion` records exactly
that intent where the lint would otherwise warn (GEP-0019-R007).

## Parameters

  - x: Any value; lists recurse, everything else is depth 0.
"""
    match _gan_args:
        case ([] as _gan_l0,) if isinstance(_gan_l0, list):
            return 1
        case (x,) if isinstance(x, list):
            _gan_tmp1 = [depth(e) for e in x]
            return 1 + builtins.max(_gan_tmp1)
        case (_,):
            return 0
    raise GanMatchError("no clause of depth/1 matched " + repr(_gan_args))


def demo() -> None:
    """Runs the chapter: constant-stack recursion, then comprehensions."""
    print(f"sum_to(1_000_000)    = {sum_to(1000000)}")
    print(f"countdown(1_000_000) = {countdown(1000000)}")
    print(f"depth([1, [2, [3]]]) = {depth([1, [2, [3]]])}")
    _gan_tmp2 = [x * x for x in [1, 2, 3, 4] if _gan_rem(x, 2) == 0]
    squares = _gan_tmp2
    _gan_tmp3 = [(x, y) for x in [1, 2] for y in [10, 20]]
    pairs = _gan_tmp3
    _gan_tmp4 = {k: v * 10 for _gan_for5 in [("a", 1), "skipped", ("b", 2)] if isinstance(_gan_for5, tuple) and len(_gan_for5) == 2 for (k, v,) in [(_gan_for5[0], _gan_for5[1],)]}
    index = _gan_tmp4
    print(f"squares = {repr(squares)}")
    print(f"pairs   = {repr(pairs)}")
    print(f"index   = {repr(index)}")
    _gan_tmp6 = [lambda *, x=x: x for x in [1, 2, 3]]
    fns = _gan_tmp6
    _gan_tmp7 = [f() for f in fns]
    return print(f"capture = {repr(_gan_tmp7)}")
