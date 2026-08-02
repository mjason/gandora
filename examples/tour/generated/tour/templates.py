"""GEP-0009: the ~<lang> embedded-language family with EEx-style
<%= expr %> splices. Any sigil name is a language tag; bodies are raw
(no #{} — it collides with target-language syntax). Splices insert
runtime values in every language, and compiled code in ~python.
"""

import builtins


def sales_query(min_units):
    return f"SELECT product, SUM(units) AS total\nFROM sales\nWHERE units >= {min_units}\nGROUP BY product\n"


def report(name, xs):
    return f"# Sales report for {name.upper()}\n\n| metric | value |\n| --- | --- |\n| count | {builtins.len(xs)} |\n| total | {builtins.sum(xs)} |\n| best  | {builtins.max(xs)} |\n"


def evens_capped(xs, limit):
    return ([x for x in (xs) if x % 2 == 0][:(limit)])


def eex_docs():
    return "Use <%= expr %> to splice a value; this line shows it literally."


def main():
    print(sales_query(10))
    print(report("gandora", [12, 30, 7, 45]))
    print(f"evens_capped = {repr(evens_capped([1, 2, 3, 4, 5, 6, 7, 8], 3))}")
    return print(eex_docs())


if __name__ == "__main__":
    main()
