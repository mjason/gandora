"""Cross-module calls, required macros, and destructuring in action."""

import builtins
import json
import time
import app.mathy


class GanMatchError(Exception):
    pass


def swap(*_gan_args) -> tuple[object, object]:
    """Destructuring in a function head: swaps a pair.

    >>> swap((1, 2))
    (2, 1)
"""
    match _gan_args:
        case ((a, b) as _gan_t0,) if isinstance(_gan_t0, tuple):
            return (b, a)
    raise GanMatchError("no clause of swap/1 matched " + repr(_gan_args))


def main() -> None:
    """Runs the chapter."""
    print(f"fact(10) = {app.mathy.fact(10)}")
    print(f"classify(-3) = {app.mathy.classify(-3)}")
    print(f"norm([3, 4]) = {app.mathy.norm([3, 4])}")
    _gan_case2 = None
    match _gan_case2:
        case None:
            _gan_tmp1 = "fallback"
        case found__gan1:
            _gan_tmp1 = found__gan1
    picked = _gan_tmp1
    print(f"unless_nil(nil, :fallback) = {picked}")
    _gan_val3 = swap((2, 1))
    match _gan_val3:
        case (a, b) as _gan_t4 if isinstance(_gan_t4, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val3))
    _gan_val5 = [10, 20, 30]
    match _gan_val5:
        case [h, *t] as _gan_l6 if isinstance(_gan_l6, list):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val5))
    print(f"destructured: {a} {b} {h} {repr(t)}")
    _gan_fstr7 = json.dumps({"lang": "gandora"}, indent=0)
    print(f"json: {_gan_fstr7}")
    print(f"sum_all([1, 2, 3, 4]) = {builtins.sum([1, 2, 3, 4])}")
    start__gan3 = time.perf_counter()
    timer_result = app.mathy.fact(200)
    elapsed__gan3 = time.perf_counter() - start__gan3
    _gan_fstr8 = "fact(200)"
    print(f"{_gan_fstr8} took {builtins.round(elapsed__gan3 * 1000, 3)} ms")
    return print(f"timer_result has {builtins.len(str(timer_result))} digits")


if __name__ == "__main__":
    main()
