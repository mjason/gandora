"""numpy: arrays, broadcasting through plain operators, and |> .method chains.
Needs `uv sync` first, then: gan run src/tour/numpy.gan
"""

import builtins
import collections.abc
import numpy as np


def norm(xs: collections.abc.Sequence[int | float]) -> float:
    """Euclidean norm of a vector — numpy scalars unwrap with $builtins.float.

## Parameters

  - xs: The vector components.

    >>> norm([3.0, 4.0])
    5.0
"""
    return builtins.float(np.linalg.norm(np.array(xs)))


def demo() -> None:
    """numpy without wrappers: arrays, broadcasting, reductions."""
    a = np.array([1.0, 2.0, 3.0, 4.0])
    print(f"a            = {a}")
    print(f"a * 10 + 1   = {(a * 10.0) + 1.0}")
    m = np.arange(12).reshape(3, 4)
    print(f"reshape(3,4) =\n{m}")
    print(f"row sums     = {m.sum(axis=1)}")
    print(f"col means    = {m.mean(axis=0)}")
    print(f"norm([3,4])  = {norm([3.0, 4.0])}")
    evens = (lambda arr: arr[arr % 2 == 0])(m)
    print(f"evens        = {evens}")
    stats = m.astype("float64").std().round(3)
    return print(f"std          = {stats}")


def main() -> None:
    """Runs the chapter."""
    return demo()


if __name__ == "__main__":
    main()
