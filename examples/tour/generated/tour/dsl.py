"""GEP-0008 metaprogramming: declaration macros generate defs, `use`
injects an API, and defattr + @on_definition build a route table —
an annotation system in userland, the way @doc itself works.
"""


def injected_greeting() -> str:
    """Injected by `use Tour.Dsl` — proof the quote landed."""
    return "hello from __using__"


def answer():
    return 42


def language():
    return "gandora"


def list_users() -> list[str]:
    """The GET handler — its @route lands in the accumulated table.

    >>> list_users()
    ['alice', 'bob']
"""
    return ["alice", "bob"]


def create_user() -> str:
    """The POST handler — same userland annotation, second entry."""
    return "created"


def routes() -> list:
    """Everything the userland annotation collected."""
    return [("get", "/users"), ("post", "/users")]


def main() -> None:
    """Runs the chapter."""
    print(injected_greeting())
    print(f"answer = {answer()}, language = {language()}")
    print(f"routes = {repr(routes())}")
    return print(f"handlers = {repr([list_users(), create_user()])}")


if __name__ == "__main__":
    main()
