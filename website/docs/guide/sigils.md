# Sigils & Embedded Languages

## The classics

```elixir
~w(one two three)            # ["one", "two", "three"]
~s(no "escaping" needed)     # string, free delimiter choice
~r/\d+/                      # re.compile(r"\d+")
```

## Two tiers, two symbols

Embedded content splits by what the body *is* (GEP-0009):

**`~<name>` is text.** Any name — `~sql`, `~markdown`, `~html`,
anything — tags a raw string template. The body passes through
byte-for-byte; `<%= expr %>` splices runtime values; editors highlight
the inner language:

```elixir
~sql"""
SELECT * FROM sales WHERE units >= <%= min_units %>
"""
~markdown(A report for <%= name %>)
```

**`$python(expr)` is code.** One verbatim Python expression entering
the program — the `$` world, same as `$math.sqrt`:

```elixir
def evens_capped(xs, limit) do
  $python([x for x in <%= xs %> if x % 2 == 0][:<%= limit %>])
end
```

Splices in `$python` are *compiled Gandora expressions*; everything
else is your Python, untouched. It is the escape hatch for
Python-only spellings — reach for it rarely.

## `~p` — AI prose without escaping

The blessed text-sigil name for prompts (GEP-0009-R006): quotes,
braces, backslashes, and inline JSON all pass through raw — never
fight `\\\"` again:

```elixir
@task ~p(Parse the JSON string {"b": 2, "a": 1} and print its keys sorted.)

@system ~p"""
You are a coding agent. Reply with {"status": "ok"} when done.
The user's name is <%= name %>.
"""
```

## Data is not embedded — it's maps

There is deliberately **no** JSON literal: Gandora maps are the one
data spelling (richer — atom keys, arbitrary expressions). A pasted
JSON document becomes a map by swapping `:` for `=>` (the build
teaches this on sight); runtime JSON text is `$json.loads(s)`.

Heredoc (`"""`) bodies dedent like ordinary heredocs, so templates
indent naturally inside modules while producing flush-left values.
