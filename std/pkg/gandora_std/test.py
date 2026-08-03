"""The official test assertions (GEP-0024): write `tests/*.gan` modules
whose `test_*` functions call these — `gan test` compiles them and
runs pytest underneath. A failure raises; pytest reports it.
"""

import builtins
import collections.abc
import gandora_std.enum
import gandora_std.map
import gandora_std.string


def _gan_truthy(value):
    return value is not None and value is not False

class GanMatchError(Exception):
    pass


def assert_eq(got: object, want: object) -> None:
    """Fails unless `got == want`, naming both sides.

## Parameters

  - got: The actual value.
  - want: The expected value.

    >>> assert_eq(1 + 1, 2)
"""
    if got != want:
        raise RuntimeError(f"expected {repr(want)}, got {repr(got)}")
    return None


def assert_true(value: object) -> None:
    """Fails unless the value is exactly `true`.

## Parameters

  - value: The boolean under test.

    >>> assert_true(2 > 1)
"""
    return assert_eq(value, True)


def assert_false(value: object) -> None:
    """Fails unless the value is exactly `false`.

## Parameters

  - value: The boolean under test.

    >>> assert_false(1 > 2)
"""
    return assert_eq(value, False)


def assert_nil(value: object) -> None:
    """Fails unless the value is `nil`.

## Parameters

  - value: The value under test.

    >>> assert_nil(gandora_std.map.get({}, "missing"))
"""
    if not ((value is None)):
        raise RuntimeError(f"expected nil, got {repr(value)}")
    return None


def assert_raises(f: collections.abc.Callable) -> str:
    """Runs `f` and fails unless it raises; returns the message.

## Parameters

  - f: A zero-arity function expected to raise.
"""
    try:
        f()
        _gan_tmp0 = "no_raise"
    except Exception as e:
        _gan_tmp0 = ("raised", str(e))
    outcome = _gan_tmp0
    _gan_case1 = outcome
    match _gan_case1:
        case "no_raise":
            raise RuntimeError("expected a raise, got none")
        case ("raised", msg) as _gan_t2 if isinstance(_gan_t2, tuple):
            return msg
        case _:
            raise GanMatchError("no case clause matched: " + repr(_gan_case1))


def assert_contains(haystack: object, needle: object) -> None:
    """Fails unless `haystack` contains `needle` (string or list).

## Parameters

  - haystack: The string or list searched.
  - needle: What must be present.

    >>> assert_contains([1, 2, 3], 2)
"""
    if isinstance(haystack, str):
        _gan_tmp3 = gandora_std.string.contains_p(haystack, needle)
    else:
        _gan_tmp3 = gandora_std.enum.member_p(haystack, needle)
    found = _gan_tmp3
    if not (_gan_truthy(found)):
        raise RuntimeError(f"expected {repr(haystack)} to contain {repr(needle)}")
    return None


def assert_op(op: str, l: object, r: object) -> None:
    _gan_case5 = op
    match _gan_case5:
        case "==":
            _gan_tmp4 = l == r
        case "!=":
            _gan_tmp4 = l != r
        case "<":
            _gan_tmp4 = l < r
        case ">":
            _gan_tmp4 = l > r
        case "<=":
            _gan_tmp4 = l <= r
        case ">=":
            _gan_tmp4 = l >= r
        case "in":
            _gan_tmp4 = gandora_std.enum.member_p(r, l)
        case _:
            raise RuntimeError(f"unknown assert op {op}")
    held = _gan_tmp4
    if not (_gan_truthy(held)):
        raise RuntimeError(f"Assertion failed: left {op} right\nleft:  {repr(l)}\nright: {repr(r)}")
    return None


def assert_truthy(value: object) -> None:
    if not (_gan_truthy(value)):
        raise RuntimeError(f"Assertion failed: expected truthy, got {repr(value)}")
    return None


def refute_truthy(value: object) -> None:
    if _gan_truthy(value):
        raise RuntimeError(f"Refute failed: expected falsy, got {repr(value)}")
    return None


def refute_in(l: object, r: object) -> None:
    if _gan_truthy(gandora_std.enum.member_p(r, l)):
        raise RuntimeError(f"Refute failed: {repr(l)} is in {repr(r)}")
    return None


def assert_raise(type: object, f: collections.abc.Callable) -> str:
    """Fails unless `f` raises an instance of `type` (ExUnit assert_raise/2).

## Parameters

  - type: The exception class, e.g. $builtins.ValueError.
  - f: A zero-arity function expected to raise it.
"""
    try:
        f()
        _gan_tmp6 = "no_raise"
    except Exception as e:
        if _gan_truthy(builtins.isinstance(e, type)):
            _gan_tmp6 = ("raised", str(e))
        else:
            _gan_tmp6 = ("wrong", builtins.type(e).__name__)
    outcome = _gan_tmp6
    _gan_case7 = outcome
    match _gan_case7:
        case "no_raise":
            raise RuntimeError(f"expected {repr(type)} to be raised, got none")
        case ("wrong", got) as _gan_t8 if isinstance(_gan_t8, tuple):
            raise RuntimeError(f"expected {repr(type)}, got {got}")
        case ("raised", msg) as _gan_t9 if isinstance(_gan_t9, tuple):
            return msg
        case _:
            raise GanMatchError("no case clause matched: " + repr(_gan_case7))


def assert_in_delta(a: int | float, b: int | float, delta: int | float) -> None:
    """Fails unless `abs(a - b) <= delta` (ExUnit assert_in_delta/3).

## Parameters

  - a: One value.
  - b: The other value.
  - delta: The allowed absolute difference.

    >>> assert_in_delta(3.14159, 3.14, 0.01)
"""
    if builtins.abs(a - b) > delta:
        raise RuntimeError(f"expected {repr(a)} and {repr(b)} to be within {repr(delta)}")
    return None


def flunk(message: str) -> None:
    """Fails immediately with the given message (ExUnit flunk/1).

## Parameters

  - message: Why the test fails.
"""
    raise RuntimeError(message)
