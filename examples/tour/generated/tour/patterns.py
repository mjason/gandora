"""Pattern matching everywhere: =, case, cond, with, guards, multi-clause heads."""

import builtins


def _gan_truthy(value):
    return value is not None and value is not False

def _gan_div(a, b):
    q = a // b
    if q < 0 and q * b != a:
        q += 1
    return q


def _gan_rem(a, b):
    return a - _gan_div(a, b) * b

class GanMatchError(Exception):
    pass


def describe(*_gan_args):
    match _gan_args:
        case (0,):
            return "zero"
        case (n,) if n < 0:
            return f"negative {n}"
        case (n,) if _gan_rem(n, 2) == 0:
            return f"even {n}"
        case (n,):
            return f"odd {n}"
    raise GanMatchError("no clause of describe/1 matched " + repr(_gan_args))


def destructure():
    _gan_val0 = ("ok", [1, 2, 3])
    match _gan_val0:
        case (status, payload) as _gan_t1 if isinstance(_gan_t1, tuple):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val0))
    _gan_val2 = payload
    match _gan_val2:
        case [first, *rest] as _gan_l3 if isinstance(_gan_l3, list):
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val2))
    _gan_val4 = {"name": "gandora", "extra": True}
    match _gan_val4:
        case {"name": name}:
            pass
        case _:
            raise GanMatchError("no match of right-hand side value: " + repr(_gan_val4))
    return (status, first, rest, name)


def handle(result, expected):
    _gan_case5 = result
    match _gan_case5:
        case ("ok", _gan_pin6) as _gan_t7 if _gan_pin6 == expected and isinstance(_gan_t7, tuple):
            return f"matched the pinned value {repr(expected)}"
        case ("ok", other) as _gan_t8 if isinstance(_gan_t8, tuple):
            return f"ok with {repr(other)}"
        case ("error", {"reason": r}) as _gan_t9 if isinstance(_gan_t9, tuple):
            return f"failed: {r}"
        case _:
            return "unknown shape"


def bucket(score):
    if score >= 90:
        return "a"
    elif score >= 60:
        return "b"
    else:
        return "c"


def parse_pair(text):
    _gan_case10 = text.split(",")
    match _gan_case10:
        case [a, b] as _gan_l11 if isinstance(_gan_l11, list):
            _gan_case12 = _parse_int(a)
            match _gan_case12:
                case ("ok", x) as _gan_t13 if isinstance(_gan_t13, tuple):
                    _gan_case14 = _parse_int(b)
                    match _gan_case14:
                        case ("ok", y) as _gan_t15 if isinstance(_gan_t15, tuple):
                            return ("ok", (x, y))
                        case _gan_with_fail__gan0:
                            _gan_case16 = _gan_with_fail__gan0
                            match _gan_case16:
                                case _:
                                    return "error"
                case _gan_with_fail__gan0:
                    _gan_case17 = _gan_with_fail__gan0
                    match _gan_case17:
                        case _:
                            return "error"
        case _gan_with_fail__gan0:
            _gan_case18 = _gan_with_fail__gan0
            match _gan_case18:
                case _:
                    return "error"


def _parse_int(s):
    if _gan_truthy(s.strip().isdigit()):
        return ("ok", builtins.int(s))
    else:
        return "error"


def demo():
    print(f"describe:    {repr([describe(0), describe(-3), describe(4), describe(7)])}")
    print(f"destructure: {repr(destructure())}")
    print(handle(("ok", 42), 42))
    print(handle(("ok", "other"), 42))
    print(handle(("error", {"reason": "boom"}), 42))
    print(f"buckets:     {repr([bucket(95), bucket(70), bucket(10)])}")
    return print(f"parse_pair:  {repr([parse_pair("3,4"), parse_pair("nope")])}")
