"""The official test assertions (GEP-0024): write `tests/*.gan` modules
whose `test_*` functions call these — `gan test` compiles them and
runs pytest underneath. A failure raises; pytest reports it.
"""

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
