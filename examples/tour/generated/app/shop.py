"""Structs + module attributes + a Flask-style decorator registry."""

import builtins
import collections.abc
import dataclasses


def _gan_div(a, b):
    q = a // b
    if q < 0 and q * b != a:
        q += 1
    return q


def _gan_rem(a, b):
    return a - _gan_div(a, b) * b


@dataclasses.dataclass(frozen=True)
class Shop:
    name: object = None
    price: object = 0
    tags: object = dataclasses.field(default_factory=lambda: [])

routes = {}


def route(path: str) -> collections.abc.Callable:
    """A Flask-style decorator factory: registers `f` under `path`.

## Parameters

  - path: The route key.
"""
    def _gan_fn0(f, *, path=path):
        routes.update({path: f})
        return f
    return _gan_fn0


@route("/sale")
def sale(item: Shop) -> Shop:
    """Half price, tagged — struct update syntax under a decorator.

## Parameters

  - item: The item on sale.
"""
    return dataclasses.replace(item, price=_gan_div(item.price, 2), tags=item.tags + ["sale"])


def main() -> None:
    """Runs the chapter."""
    item = Shop(name="keyboard", price=100)
    _gan_case0 = item
    match _gan_case0:
        case Shop(price=p) if p > 50:
            print(f"expensive: {p}")
        case _:
            print("cheap")
    handler = routes.get("/sale")
    discounted = handler(item)
    print(f"{discounted.name} now {discounted.price}, tags {repr(discounted.tags)}")
    return print(f"registered routes: {repr(builtins.list(routes.keys()))}")


if __name__ == "__main__":
    main()
