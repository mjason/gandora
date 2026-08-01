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
| [`src/tour/dataframe.gan`](tour/src/tour/dataframe.gan) | `Tour.Dataframe` | **pandas** as a `uv` dev dependency: DataFrame from a map literal, `\|> .groupby(...) \|> .agg(...)` method pipes, `.assign` with a `~python` lambda column, `.query` chains |
| [`src/tour/numpy.gan`](tour/src/tour/numpy.gan) | `Tour.Numpy` | **numpy**: broadcasting through plain operators, `\|> .reshape(3, 4)` chains, `np.linalg.norm`, `~python` boolean indexing |
| [`src/tour/dsl.gan`](tour/src/tour/dsl.gan) | `Tour.Dsl` | **GEP-0008 metaprogramming**: declaration macros, `def unquote(name)`, `use`/`__using__`, and a `defattr` + `@on_definition` route table |
| [`src/tour/templates.gan`](tour/src/tour/templates.gan) | `Tour.Templates` | **GEP-0009 embedded languages**: `~sql`/`~markdown` with `<%= expr %>` value splices, `~python` code splices, the `<%%=` escape |
| [`src/tour/annotations.gan`](tour/src/tour/annotations.gan) | `Tour.Annotations` | a native Python decorator wrapped as an Elixir-style attribute: `@cache 128` before a def applies `functools.lru_cache` via `defattr` + `@on_definition` |
| [`src/tour/web_dsl.gan`](tour/src/tour/web_dsl.gan) | `Tour.WebDsl` | a Phoenix-flavored web DSL built purely with GEP-0008 macros: `use` injects the FastAPI app, `get`/`post` macros compute handler names, splice path params, and attach decorators |
| [`src/tour/webapi.gan`](tour/src/tour/webapi.gan) | `Tour.Webapi` | **FastAPI in Elixir style**: `use Tour.WebDsl` + `get :slug, "/slug/{text}", [text] do ... end`; still serves with plain `uvicorn tour.webapi:app --app-dir dist`, self-tests with TestClient |

The pandas/numpy chapters need the dev dependencies installed and run
standalone (they are not part of `main.gan`, so the rest of the tour stays
stdlib-only):

```console
cd examples/tour
uv sync                            # installs pandas + numpy into .venv
gan run src/tour/dataframe.gan
gan run src/tour/numpy.gan
```

The method pipe (GEP-0001-R025) is what makes fluent Python APIs read as
pipelines: when the right side of `|>` starts with `.`, the call applies to
the piped value itself —

```elixir
df
|> .groupby("product", as_index: false)
|> .agg(%{"units" => "sum", "revenue" => "sum"})
|> .sort_values("revenue", ascending: false)
```

compiles to
`df.groupby("product", as_index=False).agg({...}).sort_values("revenue", ascending=False)`.

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
