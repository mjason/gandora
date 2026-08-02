"""GEP-0008 metaprogramming: declaration macros generate defs, `use`
injects an API, and defattr + @on_definition build a route table —
an annotation system in userland, the way @doc itself works.
"""


def injected_greeting():
    return "hello from __using__"


def answer():
    return 42


def language():
    return "gandora"


def list_users():
    return ["alice", "bob"]


def create_user():
    return "created"


def routes():
    return [("get", "/users"), ("post", "/users")]


def main():
    print(injected_greeting())
    print(f"answer = {answer()}, language = {language()}")
    print(f"routes = {repr(routes())}")
    return print(f"handlers = {repr([list_users(), create_user()])}")


if __name__ == "__main__":
    main()
