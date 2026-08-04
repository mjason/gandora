# Python Interop

`$module` is a first-class module object; `$` marks the interop
boundary visibly — see `$`, think Python.

```elixir
$math.sqrt(2.0)                     # import math; math.sqrt(2.0)
$importlib.metadata.version(x)      # dotted chain: imports importlib.metadata
$(PIL.Image).open(f)                # $(...) locks the module boundary explicitly
$(sys).stderr                       # ...single-segment too
pyimport numpy, as: np              # aliased import
pyimport sys                        # bare import binds `sys` as a plain name
np.array([1, 2]) * 10               # operators broadcast — it's just Python
$json.dumps(data, indent: 2)        # trailing keywords become kwargs
```

## Which spelling, when

| Situation | Spelling |
| --- | --- |
| one-off reference | `$math.sqrt(x)` |
| chain heuristic guesses wrong | `$(os.path).sep`, `$(sys).stderr` |
| module used repeatedly in a file | `pyimport sys` (or `, as:`) + bare names |
| a deep attribute used often | `@environ $(os).environ` module attribute |
| a whole Python expression | `$python(...)` — see [Sigils](sigils.md) |

Repeated `$module` in one file is a smell the build points out —
declare a `pyimport`. **Never write wrapper modules around Python
APIs**; the absence of wrappers is the design.

## Methods, pipes, exceptions

```elixir
" gan " |> .strip() |> .upper()     # method pipe on the piped value
df |> .groupby("k") |> .agg(spec)   # the pandas fluent world, one form
rescue
  e in $builtins.ValueError -> ...  # exceptions are spelled through their module
```

Python builtins live under `$builtins`: `$builtins.len(x)`,
`$builtins.round(x, 2)`, `$builtins.list(range(n))`. A dead import
named like a builtin (`$round`) gets a build error teaching the
`$builtins.` spelling.

## What the build guarantees

Interop is verified, not hoped for: the generated Python's imports and
member references are checked with ty at build time — `$requests`
without the dependency installed, `Enum.mpa`, or a wrong-arity call is
a **build error** mapped to your source line, before anything runs.
See [The Build Verdict](../tooling/build.md).
