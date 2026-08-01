# Gandora examples

## tour — the complete syntax tour

One chapter per language area. Run everything at once:

```console
cd examples/tour
gan run src/main.gan
```

or run any single chapter, e.g. `gan run src/tour/patterns.gan` after adding
a `main`, or inspect it with `gan expand` / `gan build`.

### Chapters

| File | Module | Shows |
| --- | --- | --- |
| [`src/main.gan`](tour/src/main.gan) | `Main` | aliases, captures of module functions, private helpers |
| [`src/tour/basics.gan`](tour/src/tour/basics.gan) | `Tour.Basics` | every literal, operator precedence, `//`·`div`·`rem` truncation, `<>`, `#{}` interpolation, Elixir truthiness, ranges |
| [`src/tour/patterns.gan`](tour/src/tour/patterns.gan) | `Tour.Patterns` | multi-clause `def` + `when` guards, `=` destructuring, `[h \| t]`, map patterns, pin `^x`, `case`/`cond`/`with`-`else` |
| [`src/tour/functions.gan`](tour/src/tour/functions.gan) | `Tour.Functions` | `fn` (incl. multi-clause + guards), `&(&1 + &2)`, `&:math.sqrt/1`, multi-line `\|>` pipelines |
| [`src/tour/interop.gan`](tour/src/tour/interop.gan) | `Tour.Interop` | `:module` atom calls, `pyimport ... as:`, dotted `:"os.path"`, postfix chains, kwargs, `@decorate`, attribute-held Python state |
| [`src/app/macros.gan`](tour/src/app/macros.gan) | `App.Macros` | `defmacro`, `quote`/`unquote`/`unquote_splicing`, hygiene, `var!` capture |
| [`src/app/mathy.gan`](tour/src/app/mathy.gan) | `App.Mathy` | recursion, `defp`, pipelines over interop |
| [`src/app/cli.gan`](tour/src/app/cli.gan) | `App.Cli` | `require`d macros across modules, `alias`, the `var!(timer_result)` escape in action |
| [`src/app/shop.gan`](tour/src/app/shop.gan) | `App.Shop` | `defstruct` → frozen dataclass, `%Mod{...}` literal/pattern/update, module attributes, a decorator registry (GEP-0004) |
| [`src/sigils.gan`](tour/src/sigils.gan) | `Sigils` | `~w`, `~s`, `~r`, embedded-Python `~python` (GEP-0005) |
| [`src/tour/dataframe.gan`](tour/src/tour/dataframe.gan) | `Tour.Dataframe` | **pandas** as a `uv` dev dependency: DataFrame from a map literal, `.assign` with a `~python` lambda column, groupby/agg with kwargs, `.query` chains |

The pandas chapter needs the dev dependency installed and runs standalone
(it is not part of `main.gan`, so the rest of the tour stays
stdlib-only):

```console
cd examples/tour
uv sync                            # installs pandas into .venv
gan run src/tour/dataframe.gan
```

### Reading the compilation results

[`generated/`](tour/generated/) is the checked-in output of `gan build` for
every chapter — open any `.gan` file next to its `.py` twin to see exactly
what the compiler emits. The `generated_snapshot_is_current` end-to-end test
recompiles the tour on every `cargo test` and fails if this directory drifts,
so it is always faithful to the current compiler. Refresh it after compiler
changes with:

```console
cd examples/tour && gan build && cp -r dist/* generated/
```

A taste — multi-clause dispatch with guards,
[`src/tour/patterns.gan`](tour/src/tour/patterns.gan):

```elixir
def describe(0), do: "zero"
def describe(n) when n < 0, do: "negative #{n}"
def describe(n) when rem(n, 2) == 0, do: "even #{n}"
def describe(n), do: "odd #{n}"
```

becomes [`generated/tour/patterns.py`](tour/generated/tour/patterns.py):

```python
def describe(*_gan_args):
    match _gan_args:
        case (0,):
            return "zero"
        case (n,) if n < 0:
            return f"negative {n}"
        case (n,) if _gan_rem(n, 2) == 0:
            return f"even {n}"
        case (n,):
            return f"odd {n}"
    raise GanMatchError("no clause of describe/1 matched " + repr(_gan_args))
```

Module-to-path mapping (GEP-0001-R013): `Tour.Basics` ↔
`src/tour/basics.gan` ↔ `generated/tour/basics.py`, and so on — dots become
directories, CamelCase becomes snake_case, injectively.
