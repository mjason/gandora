"""One macro: `safe(expr, fallback)` — evaluate `expr`, and if the host
raises anything, be `fallback` instead. The toolchain queries a
compiler core and a filesystem that are allowed to fail; what it is
not allowed to do is crash a verdict over it. The spelling is a
macro, not a function, so `expr` stays unevaluated until it is
guarded — and the expansion is exactly the try/rescue a reviewer
would have written by hand, in every package that imports it
(GEP-0002, GEP-0006 macro shipping).
"""

import builtins


def demo() -> tuple:
    """Proof the macro expands and guards: a crash becomes its fallback.

    >>> demo()
    ('fell back', 42)
"""
    try:
        _gan_tmp0 = builtins.int("nope")
    except Exception as _e__gan1:
        _gan_tmp0 = "fell back"
    try:
        _gan_tmp1 = builtins.int("42")
    except Exception as _e__gan2:
        _gan_tmp1 = 0
    return (_gan_tmp0, _gan_tmp1)
