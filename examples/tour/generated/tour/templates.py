"""GEP-0009: the ~<lang> embedded-language family with EEx-style
<%= expr %> splices. Any sigil name is a language tag; bodies are raw
(no #{} — it collides with target-language syntax). Splices insert
runtime values in every language, and compiled code in ~python.
"""

import builtins
import collections.abc


def sales_query(min_units: int | float) -> str:
    """A SQL query with a runtime value spliced in — ~sql is just a tag.

## Parameters

  - min_units: The cutoff spliced into the WHERE clause.
"""
    return f"SELECT product, SUM(units) AS total\nFROM sales\nWHERE units >= {min_units}\nGROUP BY product\n"


def report(name: str, xs: collections.abc.Sequence[int | float]) -> str:
    """A Markdown report — splices run any Gandora expression, pipes included.

## Parameters

  - name: The report owner, upcased in the heading.
  - xs: The sales figures.
"""
    return f"# Sales report for {name.upper()}\n\n| metric | value |\n| --- | --- |\n| count | {builtins.len(xs)} |\n| total | {builtins.sum(xs)} |\n| best  | {builtins.max(xs)} |\n"


def evens_capped(xs: collections.abc.Sequence[int], limit: int) -> list[int]:
    """The evens of `xs`, at most `limit` of them — built by a ~python splice.

## Parameters

  - xs: The candidate numbers.
  - limit: How many evens to keep.

    >>> evens_capped([1, 2, 3, 4, 5, 6, 7, 8], 3)
    [2, 4, 6]
"""
    return ([x for x in (xs) if x % 2 == 0][:(limit)])


def eex_docs() -> str:
    """`<%%=` escapes the splice marker, so templates can document templates."""
    return "Use <%= expr %> to splice a value; this line shows it literally."


def main() -> None:
    """Runs the chapter."""
    print(sales_query(10))
    print(report("gandora", [12, 30, 7, 45]))
    print(f"evens_capped = {repr(evens_capped([1, 2, 3, 4, 5, 6, 7, 8], 3))}")
    return print(eex_docs())


if __name__ == "__main__":
    main()
