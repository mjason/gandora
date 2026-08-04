# Gandora

**An Elixir-flavored language that compiles to readable Python — built
for the age of AI-written code.**

Gandora gives you Elixir's expression-everything surface — pattern
matching, pipelines, immutable bindings, macros — and compiles it to
the Python a careful reviewer would have written by hand. No runtime,
no framework: the generated code stands alone and runs anywhere Python
runs, next to numpy, pandas, FastAPI, and everything else you already
depend on.

```elixir
defmodule Stats do
  @moduledoc "Descriptive statistics over plain lists."

  @doc "The arithmetic mean, rounded to `precision` decimals."
  @spec mean(xs :: sequence(number()), precision :: integer()) :: float()
  @example """
      gan> Stats.mean([1, 2, 3, 4])
      2.5
  """
  def mean(xs, precision \\ 2) do
    $builtins.round(Enum.sum(xs) / Enum.count(xs), precision)
  end
end
```

compiles to:

```python
def mean(xs: collections.abc.Sequence[int | float], precision: int = 2) -> float:
    """The arithmetic mean, rounded to `precision` decimals. ..."""
    return round(gandora_std.enum.sum(xs) / gandora_std.enum.count(xs), precision)
```

## Why Gandora

**The compiler talks back.** `gan build` is a single verdict: compiler
errors, provable-fact warnings, best-practice advice, and *artifact
verification* — the generated Python is checked with
[ty](https://docs.astral.sh/ty/) so an undefined function, a dead
import, or a wrong-arity call is a build error with a did-you-mean,
not a runtime surprise. An idiomatic project reports **zero noise**;
the language's own standard library, toolchain, example tour, and
playground all hold that line.

**Built for AI writers, honest for human readers.** Every diagnostic
teaches the correct spelling. `gan lsc` serves the whole verdict — and
docs, symbols, references — as JSON for agents. In our recurring
evaluation, a small model that has never seen the manual converges to
green on a 24-task gauntlet purely by following the build's teaching.

**Zero runtime.** Deployment never depends on Gandora. Wheels built
from Gandora projects contain ordinary Python plus your `.gan` sources
for macro consumers — publish them to PyPI like any package.

**Recursion is safe.** Tail calls compile to `while` loops (a million
frames is fine); non-tail recursion gets a compile-time warning with
an accumulator recipe; every function's compiled shape is visible on
hover.

## The pieces

| | |
| --- | --- |
| `ganc` | the stage-0 compiler (Rust, zero dependencies) |
| `gan` | the task runner — build · run · test · fmt · repl (written in Gandora) |
| `gan-lsp` / `gan lsc` | the language server and its JSON console (written in Gandora) |
| `Enum` `Map` `List` `Keyword` `String` `Test` | the standard library — every function documented, typed, and doctested |

Start with [Getting Started](getting-started.md), skim the
[Guide](guide/modules.md), or point your agent at
[Writing Gandora with AI](ai.md).
