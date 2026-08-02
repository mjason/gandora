"""Cross-module calls, required macros, and destructuring in action."""

import builtins
import json
import time
import app.mathy


class GanMatchError(Exception):
    pass


def main():
    print(f"fact(10) = {app.mathy.fact(10)}")
    print(f"classify(-3) = {app.mathy.classify(-3)}")
    print(f"norm([3, 4]) = {app.mathy.norm([3, 4])}")
    _gan_case1 = None
    match _gan_case1:
        case None:
            _gan_tmp0 = "fallback"
        case found__gan1:
            _gan_tmp0 = found__gan1
    picked = _gan_tmp0
    print(f"unless_nil(nil, :fallback) = {picked}")
    _gan_val2 = (1, 2)
    match _gan_val2:
        case (a, b) as _gan_t3 if isinstance(_gan_t3, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val2))
    _gan_val4 = [10, 20, 30]
    match _gan_val4:
        case [h, *t] as _gan_l5 if isinstance(_gan_l5, list):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val4))
    print(f"destructured: {a} {b} {h} {repr(t)}")
    _gan_fstr6 = json.dumps({"lang": "gandora"}, indent=0)
    print(f"json: {_gan_fstr6}")
    print(f"sum_all([1, 2, 3, 4]) = {builtins.sum([1, 2, 3, 4])}")
    start__gan3 = time.perf_counter()
    timer_result = app.mathy.fact(200)
    elapsed__gan3 = time.perf_counter() - start__gan3
    _gan_fstr7 = "fact(200)"
    print(f"{_gan_fstr7} took {builtins.round(elapsed__gan3 * 1000, 3)} ms")
    return print(f"timer_result has {builtins.len(str(timer_result))} digits")


if __name__ == "__main__":
    main()
