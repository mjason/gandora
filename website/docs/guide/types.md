# The Type System

`@spec` declares a function's type; the compiler validates it against
the clauses and emits **PEP 484 annotations** in the generated Python,
so `pyright`/`mypy`/ty check callers and hover shows real types. One
`@spec` per definition group, placed with the other annotations before
the first clause.

```elixir
@spec mean(xs :: sequence(number()), precision :: integer()) :: float()
def mean(xs, precision \\ 2) do ... end
# -> def mean(xs: collections.abc.Sequence[int | float], precision: int = 2) -> float:
```

## Scalars

| Gandora | Python annotation |
| --- | --- |
| `integer()` | `int` |
| `float()` | `float` |
| `number()` | `int \| float` |
| `string()` | `str` |
| `boolean()` | `bool` |
| `atom()` | `str` (atoms are interned strings) |
| `nil` | `None` |
| `term()` | `object` |
| `fun()` | `Callable` (unparametrized) |

## Containers — abstract in, concrete out

Concrete containers say "exactly this Python type":

```elixir
list(integer())            # list[int]
tuple(atom(), string())    # tuple[str, str]
map(string(), integer())   # dict[str, int]
```

Abstract containers say "anything that behaves like one" — prefer them
for **parameters** so callers may pass tuples, ranges, or generators
(`list` is invariant in Python's type system; `Sequence` is
covariant):

```elixir
iterable(t)                # collections.abc.Iterable[t]  — walked once
sequence(t)                # collections.abc.Sequence[t]  — indexed / re-walked
mapping(k, v)              # collections.abc.Mapping[k, v] — read-only dict-like
keyword()                  # a keyword list: list[tuple[str, object]]
```

Rule of thumb: **accept `sequence(t)`, return `list(t)`.** The build's
practice pass reminds you when a parameter is concrete.

## Unions, type variables, named parameters

```elixir
@spec find(sequence(a), fun()) :: a | nil     # -> _T_a | None
@spec get(map(k, v), k, v) :: v               # 1–2 letter names are TypeVars
@spec slice(xs :: sequence(a), start :: integer(), count :: integer()) :: list(a)
```

The same letter means "the same type" across a spec. Longer bare names
are an error with a targeted hint (`Int` → "write `integer()`";
`{:ok, x}` tuple literals → "documents as `tuple(atom(), x)`") — a
typo cannot silently become a generic.

## Host (Python) and struct types

```elixir
@spec sales() :: $pandas.DataFrame()
@spec run(list(string())) :: $subprocess.CompletedProcess(string())
@spec sale(App.Shop.t()) :: App.Shop.t()   # struct class from defstruct
```

The imports are emitted with the annotation.

## Specs cannot rot

A `@spec` naming a function or arity that doesn't exist is a compile
error; one spec covers a whole default-parameter group, and the
generated signatures carry the annotations — so `gan build --strict`
(full ty type-flow) always has something honest to check.
